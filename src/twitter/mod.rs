pub mod types;

use std::env::{consts, var};

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest::{
    header::HeaderMap,
    multipart::{Form, Part},
    Body,
};
use reqwest_oauth1::{OAuthClientProvider, Secrets};

use crate::POST_INTERVAL_SECS;

use self::types::{
    BaseResponse, InitMediaResponse, MeUser, Tweet, TweetMedia, TweetRequest, TweetResponse,
};

const TWITTER_API_BASE_URL: &str = "https://api.twitter.com";
const TWITTER_MEDIA_BASE_URL: &str = "https://upload.twitter.com";

pub struct TwitterClient<'a> {
    client: reqwest::Client,
    secrets: Secrets<'a>,
}

impl<'a> TwitterClient<'a> {
    pub fn new() -> Result<Self> {
        let secrets = Secrets::new_with_token(
            var("TWITTER_CONSUMER_KEY").with_context(|| "TWITTER_CONSUMER_KEY")?,
            var("TWITTER_CONSUMER_SECRET").with_context(|| "TWITTER_CONSUMER_SECRET")?,
            var("TWITTER_ACCESS_TOKEN").with_context(|| "TWITTER_ACCESS_TOKEN")?,
            var("TWITTER_ACCESS_TOKEN_SECRET").with_context(|| "TWITTER_ACCESS_TOKEN_SECRET")?,
        );

        let mut headers = HeaderMap::new();

        headers.insert("accept", "application/json".parse()?);
        headers.insert(
            "user-agent",
            format!(
                "{}/{} on {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                consts::OS
            )
            .parse()?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client, secrets })
    }

    async fn twitter_request<T>(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
        data: Option<(&str, Body)>,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{TWITTER_API_BASE_URL}{path}")
        };

        let mut request = self
            .client
            .clone()
            .oauth1(self.secrets.clone())
            .request(method.parse()?, url)
            .query(query);

        if let Some((content_type, body)) = data {
            request = request.header("content-type", content_type).body(body);
        }

        let response = match request.send().await? {
            response if response.status().is_success() => response,
            response => {
                let status = response.status();
                let error = response.text().await?;

                bail!("Twitter API error: {status} {error}");
            }
        };

        let bytes = response.bytes().await?;

        match serde_json::from_slice(&bytes) {
            Ok(base) => Ok(base),
            Err(_) => {
                let error = String::from_utf8_lossy(&bytes);

                Err(anyhow!("Failed to parse response: {error}"))
            }
        }
    }

    /// Get the authenticated user's profile.
    pub async fn get_me(&self) -> Result<MeUser> {
        self.twitter_request::<BaseResponse<_>>(
            "GET",
            "/2/users/me",
            &[("user.fields", "id,username")],
            None,
        )
        .await
        .map(|r| r.data)
    }

    /// Get the latest tweet from the given user.
    pub async fn last_tweet_ts(&self, user_id: &str) -> Result<DateTime<Utc>> {
        let tweets = self
            .twitter_request::<BaseResponse<Option<Vec<Tweet>>>>(
                "GET",
                &format!("/2/users/{user_id}/tweets"),
                &[("tweet.fields", "created_at")],
                None,
            )
            .await
            .map(|r| r.data)?
            .unwrap_or_default();

        if let Some(timestamp) = tweets.first().map(|t| &t.created_at) {
            return Ok(*timestamp);
        }

        Ok(Utc::now() - Duration::seconds(POST_INTERVAL_SECS))
    }

    /// Upload a media and return the media ID.
    pub async fn upload_media(
        &self,
        content_type: &str,
        part: Part,
        total_bytes: u64,
    ) -> Result<String> {
        let media_id = self
            .twitter_request::<InitMediaResponse>(
                "POST",
                &format!("{TWITTER_MEDIA_BASE_URL}/1.1/media/upload.json"),
                &[
                    ("command", "INIT"),
                    ("total_bytes", &total_bytes.to_string()),
                    ("media_type", content_type),
                ],
                None,
            )
            .await?
            .media_id;

        let form = Form::new().part("media", part);

        self.client
            .clone()
            .oauth1(self.secrets.clone())
            .request(
                "POST".parse()?,
                &format!(
                    "{TWITTER_MEDIA_BASE_URL}/1.1/media/upload.json?command=APPEND&media_id={media_id}&segment_index=0",
                ),
            )
            .multipart(form)
            .send()
            .await?.error_for_status()?;

        self.twitter_request::<InitMediaResponse>(
            "POST",
            &format!(
                "{TWITTER_MEDIA_BASE_URL}/1.1/media/upload.json?command=FINALIZE&media_id={media_id}",
            ),
            &[],
            None,
        )
        .await?;

        Ok(media_id)
    }

    /// Post a tweet with the given media.
    /// Returns the tweet ID.
    pub async fn post_tweet(&self, media_id: &str) -> Result<String> {
        let tweet = self
            .twitter_request::<BaseResponse<TweetResponse>>(
                "POST",
                "/2/tweets",
                &[],
                Some((
                    "application/json",
                    serde_json::to_vec(&TweetRequest {
                        text: "#capybara".to_string(),
                        media: TweetMedia {
                            media_ids: vec![media_id.to_string()],
                        },
                    })?
                    .into(),
                )),
            )
            .await
            .map(|r| r.data)?;

        Ok(tweet.id)
    }
}
