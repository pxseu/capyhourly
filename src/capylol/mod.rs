pub mod types;

use anyhow::{anyhow, Result};

const CAPY_API_BASE_URL: &str = "https://api.capy.lol";

pub struct CapyLol {
    client: reqwest::Client,
}

impl Default for CapyLol {
    fn default() -> Self {
        Self::new()
    }
}

impl CapyLol {
    pub fn new() -> Self {
        CapyLol {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_random_image(&self) -> Result<(String, Vec<u8>)> {
        let image = self
            .client
            .request("GET".parse()?, &format!("{CAPY_API_BASE_URL}/v1/capybara"))
            .send()
            .await?
            .error_for_status()?;

        let content_type = image
            .headers()
            .get("content-type")
            .ok_or_else(|| anyhow!("Missing content-type header"))?
            .to_str()?
            .to_string();

        let bytes = image.bytes().await?;

        Ok((content_type, bytes.to_vec()))
    }
}
