use reqwest::Client;
use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct ServerInfo {
    pub status: String,
    #[serde(default)]
    pub versions: serde_json::Value,
}

pub struct ObsidianClient {
    http: Client,
    base_url: String,
    api_key: String,
}

impl ObsidianClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        let http = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("failed to build HTTP client");

        Self {
            http,
            base_url,
            api_key,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    pub async fn server_info(&self) -> Result<ServerInfo, AppError> {
        let resp = self
            .http
            .get(self.url("/"))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.json().await?)
    }
}
