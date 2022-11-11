use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct MeUser {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct BaseResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct Tweet {
    pub id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct InitMediaResponse {
    #[serde(rename = "media_id_string")]
    pub media_id: String,
    pub expires_after_secs: u64,
}

#[derive(Debug, Serialize)]
pub struct TweetRequest {
    pub text: String,
    pub media: TweetMedia,
}

#[derive(Debug, Serialize)]
pub struct TweetMedia {
    pub media_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TweetResponse {
    pub id: String,
}
