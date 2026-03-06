use reqwest::Client;
use serde::Deserialize;

use crate::error::AppError;
use crate::types::PatchParams;

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct ServerInfo {
    pub status: String,
    #[serde(default)]
    pub versions: serde_json::Value,
}

pub struct ObsidianClient {
    http: Client,
    base_url: String,
    bearer_token: String,
}

impl ObsidianClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        let http = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("failed to build HTTP client");

        let bearer_token = format!("Bearer {}", api_key);

        Self {
            http,
            base_url,
            bearer_token,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn check_response(&self, resp: reqwest::Response) -> Result<reqwest::Response, AppError> {
        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }
        Ok(resp)
    }

    pub async fn server_info(&self) -> Result<ServerInfo, AppError> {
        let resp = self
            .http
            .get(self.url("/"))
            .header("Authorization", &self.bearer_token)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn read_note(&self, path: &str) -> Result<String, AppError> {
        let resp = self
            .http
            .get(self.url(&format!("/vault/{}", path)))
            .header("Authorization", &self.bearer_token)
            .header("Accept", "text/markdown")
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.text().await?)
    }

    pub async fn create_note(&self, path: &str, content: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .put(self.url(&format!("/vault/{}", path)))
            .header("Authorization", &self.bearer_token)
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
            .send()
            .await?;
        self.check_response(resp).await?;
        Ok(())
    }

    pub async fn append_note(&self, path: &str, content: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .post(self.url(&format!("/vault/{}", path)))
            .header("Authorization", &self.bearer_token)
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
            .send()
            .await?;
        self.check_response(resp).await?;
        Ok(())
    }

    pub async fn patch_note(
        &self,
        path: &str,
        params: &PatchParams,
        content: &str,
    ) -> Result<String, AppError> {
        let mut req = self
            .http
            .patch(self.url(&format!("/vault/{}", path)))
            .header("Authorization", &self.bearer_token)
            .header("Content-Type", "text/markdown")
            .header("Operation", params.operation.to_string())
            .header("Target-Type", params.target_type.to_string())
            .header("Target", &params.target);

        if let Some(ref delimiter) = params.target_delimiter {
            req = req.header("Target-Delimiter", delimiter);
        }
        if let Some(trim) = params.trim_target_whitespace {
            req = req.header("Trim-Target-Whitespace", trim.to_string());
        }
        if let Some(create) = params.create_target_if_missing {
            req = req.header("Create-Target-If-Missing", create.to_string());
        }

        let resp = req.body(content.to_string()).send().await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.text().await?)
    }

    pub async fn delete_note(&self, path: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .delete(self.url(&format!("/vault/{}", path)))
            .header("Authorization", &self.bearer_token)
            .send()
            .await?;
        self.check_response(resp).await?;
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
            .header("Authorization", &self.bearer_token)
            .header("Accept", "application/json")
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn search_simple(&self, query: &str) -> Result<serde_json::Value, AppError> {
        let resp = self
            .http
            .post(self.url("/search/simple/"))
            .header("Authorization", &self.bearer_token)
            .header("Content-Type", "text/plain")
            .body(query.to_string())
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn search_query(&self, query: &str) -> Result<serde_json::Value, AppError> {
        let resp = self
            .http
            .post(self.url("/search/"))
            .header("Authorization", &self.bearer_token)
            .header("Content-Type", "application/vnd.olrapi.dataview.dql+txt")
            .body(query.to_string())
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn list_commands(&self) -> Result<serde_json::Value, AppError> {
        let resp = self
            .http
            .get(self.url("/commands/"))
            .header("Authorization", &self.bearer_token)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn execute_command(&self, command_id: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .post(self.url(&format!("/commands/{}/", command_id)))
            .header("Authorization", &self.bearer_token)
            .send()
            .await?;
        self.check_response(resp).await?;
        Ok(())
    }

    pub async fn open_file(&self, filename: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .post(self.url(&format!("/open/{}", filename)))
            .header("Authorization", &self.bearer_token)
            .send()
            .await?;
        self.check_response(resp).await?;
        Ok(())
    }

    fn periodic_url(
        &self,
        period: &str,
        year: Option<u32>,
        month: Option<u32>,
        day: Option<u32>,
    ) -> String {
        match (year, month, day) {
            (Some(y), Some(m), Some(d)) => {
                self.url(&format!("/periodic/{}/{}/{}/{}/", period, y, m, d))
            }
            _ => self.url(&format!("/periodic/{}/", period)),
        }
    }

    pub async fn get_periodic_note(
        &self,
        period: &str,
        year: Option<u32>,
        month: Option<u32>,
        day: Option<u32>,
    ) -> Result<String, AppError> {
        let resp = self
            .http
            .get(self.periodic_url(period, year, month, day))
            .header("Authorization", &self.bearer_token)
            .header("Accept", "text/markdown")
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.text().await?)
    }

    pub async fn update_periodic_note(
        &self,
        period: &str,
        year: Option<u32>,
        month: Option<u32>,
        day: Option<u32>,
        content: &str,
    ) -> Result<(), AppError> {
        let resp = self
            .http
            .put(self.periodic_url(period, year, month, day))
            .header("Authorization", &self.bearer_token)
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
            .send()
            .await?;
        self.check_response(resp).await?;
        Ok(())
    }

    pub async fn append_periodic_note(
        &self,
        period: &str,
        year: Option<u32>,
        month: Option<u32>,
        day: Option<u32>,
        content: &str,
    ) -> Result<(), AppError> {
        let resp = self
            .http
            .post(self.periodic_url(period, year, month, day))
            .header("Authorization", &self.bearer_token)
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
            .send()
            .await?;
        self.check_response(resp).await?;
        Ok(())
    }

    pub async fn patch_periodic_note(
        &self,
        period: &str,
        year: Option<u32>,
        month: Option<u32>,
        day: Option<u32>,
        params: &PatchParams,
        content: &str,
    ) -> Result<String, AppError> {
        let mut req = self
            .http
            .patch(self.periodic_url(period, year, month, day))
            .header("Authorization", &self.bearer_token)
            .header("Content-Type", "text/markdown")
            .header("Operation", params.operation.to_string())
            .header("Target-Type", params.target_type.to_string())
            .header("Target", &params.target);

        if let Some(ref delimiter) = params.target_delimiter {
            req = req.header("Target-Delimiter", delimiter);
        }
        if let Some(trim) = params.trim_target_whitespace {
            req = req.header("Trim-Target-Whitespace", trim.to_string());
        }
        if let Some(create) = params.create_target_if_missing {
            req = req.header("Create-Target-If-Missing", create.to_string());
        }

        let resp = req.body(content.to_string()).send().await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.text().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Operation, PatchParams, TargetType};
    use wiremock::matchers::{body_string, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_client() -> ObsidianClient {
        ObsidianClient::new(
            "https://localhost:27124".to_string(),
            "test-api-key".to_string(),
        )
    }

    fn mock_client(uri: String) -> ObsidianClient {
        ObsidianClient::new(uri, "test-key".to_string())
    }

    #[test]
    fn url_concatenates_base_and_path() {
        let client = make_client();
        assert_eq!(
            client.url("/vault/test.md"),
            "https://localhost:27124/vault/test.md"
        );
    }

    #[test]
    fn bearer_token_formats_correctly() {
        let client = make_client();
        assert_eq!(client.bearer_token, "Bearer test-api-key");
    }

    #[test]
    fn periodic_url_with_all_date_params() {
        let client = make_client();
        let url = client.periodic_url("daily", Some(2026), Some(3), Some(6));
        assert_eq!(url, "https://localhost:27124/periodic/daily/2026/3/6/");
    }

    #[test]
    fn periodic_url_without_date_params() {
        let client = make_client();
        let url = client.periodic_url("weekly", None, None, None);
        assert_eq!(url, "https://localhost:27124/periodic/weekly/");
    }

    #[test]
    fn periodic_url_with_partial_date_falls_back_to_period_only() {
        let client = make_client();
        // year only, no month/day
        let url = client.periodic_url("monthly", Some(2026), None, None);
        assert_eq!(url, "https://localhost:27124/periodic/monthly/");

        // year and month, no day
        let url = client.periodic_url("daily", Some(2026), Some(3), None);
        assert_eq!(url, "https://localhost:27124/periodic/daily/");
    }

    // ---- wiremock-based integration tests ----

    #[tokio::test]
    async fn server_info_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"status": "OK", "versions": {}})),
            )
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let info = client.server_info().await.unwrap();
        assert_eq!(info.status, "OK");
    }

    #[tokio::test]
    async fn read_note_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vault/folder/note.md"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Accept", "text/markdown"))
            .respond_with(ResponseTemplate::new(200).set_body_string("# Hello"))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let result = client.read_note("folder/note.md").await.unwrap();
        assert_eq!(result, "# Hello");
    }

    #[tokio::test]
    async fn create_note_sends_put_with_markdown() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/vault/new.md"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Content-Type", "text/markdown"))
            .and(body_string("# New note"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        client.create_note("new.md", "# New note").await.unwrap();
    }

    #[tokio::test]
    async fn append_note_sends_post_with_markdown() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/vault/existing.md"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Content-Type", "text/markdown"))
            .and(body_string("appended content"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        client
            .append_note("existing.md", "appended content")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn patch_note_sends_v3_headers() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/vault/note.md"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Content-Type", "text/markdown"))
            .and(header("Operation", "append"))
            .and(header("Target-Type", "heading"))
            .and(header("Target", "Introduction"))
            .and(body_string("new content"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let params = PatchParams {
            operation: Operation::Append,
            target_type: TargetType::Heading,
            target: "Introduction".to_string(),
            target_delimiter: None,
            trim_target_whitespace: None,
            create_target_if_missing: None,
        };
        let result = client
            .patch_note("note.md", &params, "new content")
            .await
            .unwrap();
        assert_eq!(result, "ok");
    }

    #[tokio::test]
    async fn patch_note_sends_optional_headers() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/vault/note.md"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Content-Type", "text/markdown"))
            .and(header("Operation", "replace"))
            .and(header("Target-Type", "frontmatter"))
            .and(header("Target", "tags"))
            .and(header("Target-Delimiter", "/"))
            .and(header("Trim-Target-Whitespace", "true"))
            .and(header("Create-Target-If-Missing", "true"))
            .and(body_string("new-tag"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let params = PatchParams {
            operation: Operation::Replace,
            target_type: TargetType::Frontmatter,
            target: "tags".to_string(),
            target_delimiter: Some("/".to_string()),
            trim_target_whitespace: Some(true),
            create_target_if_missing: Some(true),
        };
        let result = client
            .patch_note("note.md", &params, "new-tag")
            .await
            .unwrap();
        assert_eq!(result, "ok");
    }

    #[tokio::test]
    async fn delete_note_sends_delete() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/vault/old.md"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        client.delete_note("old.md").await.unwrap();
    }

    #[tokio::test]
    async fn list_files_root() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vault/"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Accept", "application/json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"files": ["a.md", "b.md"]})),
            )
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let result = client.list_files(None).await.unwrap();
        assert_eq!(result, serde_json::json!({"files": ["a.md", "b.md"]}));
    }

    #[tokio::test]
    async fn list_files_subdirectory() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vault/subdir/"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Accept", "application/json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"files": ["subdir/c.md"]})),
            )
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let result = client.list_files(Some("subdir")).await.unwrap();
        assert_eq!(result, serde_json::json!({"files": ["subdir/c.md"]}));
    }

    #[tokio::test]
    async fn search_simple_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search/simple/"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Content-Type", "text/plain"))
            .and(body_string("my query"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!([{"filename": "a.md"}])),
            )
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let result = client.search_simple("my query").await.unwrap();
        assert_eq!(result, serde_json::json!([{"filename": "a.md"}]));
    }

    #[tokio::test]
    async fn search_query_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search/"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header(
                "Content-Type",
                "application/vnd.olrapi.dataview.dql+txt",
            ))
            .and(body_string("TABLE file.name FROM #tag"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"results": []})),
            )
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let result = client
            .search_query("TABLE file.name FROM #tag")
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!({"results": []}));
    }

    #[tokio::test]
    async fn list_commands_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/commands/"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"commands": [{"id": "cmd1", "name": "Command 1"}]}),
            ))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let result = client.list_commands().await.unwrap();
        assert_eq!(
            result,
            serde_json::json!({"commands": [{"id": "cmd1", "name": "Command 1"}]})
        );
    }

    #[tokio::test]
    async fn execute_command_sends_post() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/commands/editor:toggle-bold/"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        client.execute_command("editor:toggle-bold").await.unwrap();
    }

    #[tokio::test]
    async fn open_file_sends_post() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/open/my-note.md"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        client.open_file("my-note.md").await.unwrap();
    }

    #[tokio::test]
    async fn get_periodic_note_without_date() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/periodic/daily/"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Accept", "text/markdown"))
            .respond_with(ResponseTemplate::new(200).set_body_string("# Daily Note"))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let result = client
            .get_periodic_note("daily", None, None, None)
            .await
            .unwrap();
        assert_eq!(result, "# Daily Note");
    }

    #[tokio::test]
    async fn get_periodic_note_with_date() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/periodic/daily/2026/3/6/"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Accept", "text/markdown"))
            .respond_with(ResponseTemplate::new(200).set_body_string("# 2026-03-06"))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let result = client
            .get_periodic_note("daily", Some(2026), Some(3), Some(6))
            .await
            .unwrap();
        assert_eq!(result, "# 2026-03-06");
    }

    #[tokio::test]
    async fn update_periodic_note_sends_put() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/periodic/weekly/2026/3/6/"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Content-Type", "text/markdown"))
            .and(body_string("weekly content"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        client
            .update_periodic_note("weekly", Some(2026), Some(3), Some(6), "weekly content")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn append_periodic_note_sends_post() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/periodic/daily/2026/3/6/"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Content-Type", "text/markdown"))
            .and(body_string("appended"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        client
            .append_periodic_note("daily", Some(2026), Some(3), Some(6), "appended")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn patch_periodic_note_sends_v3_headers() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/periodic/daily/2026/3/6/"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Content-Type", "text/markdown"))
            .and(header("Operation", "append"))
            .and(header("Target-Type", "heading"))
            .and(header("Target", "Tasks"))
            .and(body_string("- [ ] do thing"))
            .respond_with(ResponseTemplate::new(200).set_body_string("patched daily"))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let params = PatchParams {
            operation: Operation::Append,
            target_type: TargetType::Heading,
            target: "Tasks".to_string(),
            target_delimiter: None,
            trim_target_whitespace: None,
            create_target_if_missing: None,
        };
        let result = client
            .patch_periodic_note(
                "daily",
                Some(2026),
                Some(3),
                Some(6),
                &params,
                "- [ ] do thing",
            )
            .await
            .unwrap();
        assert_eq!(result, "patched daily");
    }

    #[tokio::test]
    async fn patch_periodic_note_without_date() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/periodic/monthly/"))
            .and(header("Authorization", "Bearer test-key"))
            .and(header("Content-Type", "text/markdown"))
            .and(header("Operation", "replace"))
            .and(header("Target-Type", "block"))
            .and(header("Target", "abc123"))
            .and(body_string("replaced"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let params = PatchParams {
            operation: Operation::Replace,
            target_type: TargetType::Block,
            target: "abc123".to_string(),
            target_delimiter: None,
            trim_target_whitespace: None,
            create_target_if_missing: None,
        };
        let result = client
            .patch_periodic_note("monthly", None, None, None, &params, "replaced")
            .await
            .unwrap();
        assert_eq!(result, "ok");
    }

    // ---- error case (one representative test) ----

    #[tokio::test]
    async fn api_error_on_non_success_status() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vault/missing.md"))
            .respond_with(ResponseTemplate::new(404).set_body_string("File not found"))
            .mount(&server)
            .await;

        let client = mock_client(server.uri());
        let err = client.read_note("missing.md").await.unwrap_err();
        match err {
            AppError::Api { status, body } => {
                assert_eq!(status, 404);
                assert_eq!(body, "File not found");
            }
            other => panic!("expected AppError::Api, got: {:?}", other),
        }
    }
}
