use anyhow::{bail, Result};
use capyhourly::{capylol::CapyLol, twitter::TwitterClient};
use reqwest::multipart::Part;
use tokio::time::{interval, sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let twitter = TwitterClient::new()?;
    let capy = CapyLol::new();

    let me = twitter.get_me().await?;
    println!("Logged in as @{} ({})", me.username, me.id);

    let last_tweet = twitter.last_tweet_ts(&me.id).await?;
    println!("Last tweet on: {last_tweet}");

    let duration =
        last_tweet + chrono::Duration::seconds(capyhourly::POST_INTERVAL_SECS) - chrono::Utc::now();

    if duration > chrono::Duration::zero() {
        println!("Waiting untill next tweet can be posted...");

        sleep(duration.to_std()?).await;
    }

    let mut interval = interval(Duration::from_secs(capyhourly::POST_INTERVAL_SECS as u64));

    loop {
        interval.tick().await;

        println!("Getting image");

        let (content_type, image) = capy.get_random_image().await?;

        println!("Uploading image");

        let ext = match content_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            _ => bail!("Unsupported media type: `{content_type}`"),
        };

        let size = image.len() as u64;
        let part = Part::bytes(image).file_name(format!("capybara.{ext}"));

        let media_id = twitter.upload_media(&content_type, part, size).await?;

        println!("Posting tweet...");

        let tweet = twitter.post_tweet(&media_id).await?;

        println!("Posted tweet: {tweet}");
    }
}
