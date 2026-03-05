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

    pub async fn read_note(&self, path: &str) -> Result<String, AppError> {
        let resp = self
            .http
            .get(self.url(&format!("/vault/{}", path)))
            .header("Authorization", self.auth_header())
            .header("Accept", "text/markdown")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.text().await?)
    }

    pub async fn create_note(&self, path: &str, content: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .put(self.url(&format!("/vault/{}", path)))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    pub async fn append_note(&self, path: &str, content: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .post(self.url(&format!("/vault/{}", path)))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    pub async fn patch_note(
        &self,
        path: &str,
        heading: Option<&str>,
        content: &str,
    ) -> Result<String, AppError> {
        let mut req = self
            .http
            .patch(self.url(&format!("/vault/{}", path)))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown");

        if let Some(heading) = heading {
            req = req.header("X-Heading", heading);
        }

        let resp = req.body(content.to_string()).send().await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.text().await?)
    }

    pub async fn delete_note(&self, path: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .delete(self.url(&format!("/vault/{}", path)))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    pub async fn list_files(&self, path: Option<&str>) -> Result<serde_json::Value, AppError> {
        let url = match path {
            Some(p) => self.url(&format!("/vault/{}/", p)),
            None => self.url("/vault/"),
        };

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
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

    pub async fn search_simple(&self, query: &str) -> Result<serde_json::Value, AppError> {
        let resp = self
            .http
            .post(self.url("/search/simple/"))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/plain")
            .body(query.to_string())
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

    pub async fn search_query(&self, query: &str) -> Result<serde_json::Value, AppError> {
        let resp = self
            .http
            .post(self.url("/search/"))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/vnd.olrapi.dataview.dql+txt")
            .body(query.to_string())
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

    pub async fn list_commands(&self) -> Result<serde_json::Value, AppError> {
        let resp = self
            .http
            .get(self.url("/commands/"))
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

    pub async fn execute_command(&self, command_id: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .post(self.url(&format!("/commands/{}/", command_id)))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    pub async fn open_file(&self, filename: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .post(self.url(&format!("/open/{}", filename)))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }
}
