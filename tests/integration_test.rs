use std::sync::Arc;

use obsidian_mcp::client::ObsidianClient;
use obsidian_mcp::server::ObsidianServer;
use rmcp::ServerHandler;
use wiremock::matchers::{body_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Create an ObsidianClient backed by a wiremock server.
fn mock_client(uri: String) -> ObsidianClient {
    ObsidianClient::new(uri, "test-key".to_string())
}

/// Create a full ObsidianServer backed by a wiremock mock.
async fn setup() -> (MockServer, ObsidianServer) {
    let mock = MockServer::start().await;
    let client = Arc::new(mock_client(mock.uri()));
    let server = ObsidianServer::new(client);
    (mock, server)
}

// ---------------------------------------------------------------------------
// Tests that exercise ObsidianClient through the public library API,
// proving the lib.rs extraction works and the full HTTP round-trip via wiremock.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_read_note_end_to_end() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/my/note.md"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Accept", "text/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_string("# My Note\n\nContent here"))
        .mount(&mock)
        .await;

    let client = mock_client(mock.uri());
    let content = client.read_note("my/note.md").await.unwrap();
    assert_eq!(content, "# My Note\n\nContent here");
}

#[tokio::test]
async fn client_create_note_end_to_end() {
    let mock = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/vault/new/doc.md"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "text/markdown"))
        .and(body_string("# Fresh"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let client = mock_client(mock.uri());
    client.create_note("new/doc.md", "# Fresh").await.unwrap();
}

#[tokio::test]
async fn client_search_end_to_end() {
    let mock = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/search/simple/"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "text/plain"))
        .and(body_string("rust programming"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([{"filename": "rust.md", "score": 0.95}])),
        )
        .mount(&mock)
        .await;

    let client = mock_client(mock.uri());
    let result = client.search_simple("rust programming").await.unwrap();
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["filename"], "rust.md");
}

#[tokio::test]
async fn client_execute_command_end_to_end() {
    let mock = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/commands/editor:toggle-bold/"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let client = mock_client(mock.uri());
    client
        .execute_command("editor:toggle-bold")
        .await
        .unwrap();
}

#[tokio::test]
async fn client_returns_error_on_api_failure() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/missing.md"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&mock)
        .await;

    let client = mock_client(mock.uri());
    let err = client.read_note("missing.md").await.unwrap_err();
    match err {
        obsidian_mcp::error::AppError::Api { status, body } => {
            assert_eq!(status, 404);
            assert_eq!(body, "not found");
        }
        other => panic!("expected AppError::Api, got: {:?}", other),
    }
}

#[tokio::test]
async fn client_periodic_note_end_to_end() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/periodic/daily/"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Accept", "text/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_string("# Today\n\n- Task 1"))
        .mount(&mock)
        .await;

    let client = mock_client(mock.uri());
    let content = client
        .get_periodic_note("daily", None, None, None)
        .await
        .unwrap();
    assert!(content.contains("Task 1"));
}

#[tokio::test]
async fn client_append_note_end_to_end() {
    let mock = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/vault/journal.md"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "text/markdown"))
        .and(body_string("## Evening\nReflections"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock)
        .await;

    let client = mock_client(mock.uri());
    client
        .append_note("journal.md", "## Evening\nReflections")
        .await
        .unwrap();
}

#[tokio::test]
async fn client_delete_note_end_to_end() {
    let mock = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/vault/old-note.md"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let client = mock_client(mock.uri());
    client.delete_note("old-note.md").await.unwrap();
}

#[tokio::test]
async fn client_list_files_end_to_end() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Accept", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"files": ["a.md", "b.md", "sub/c.md"]})),
        )
        .mount(&mock)
        .await;

    let client = mock_client(mock.uri());
    let result = client.list_files(None).await.unwrap();
    let files = result["files"].as_array().unwrap();
    assert_eq!(files.len(), 3);
}

// ---------------------------------------------------------------------------
// Verify that ObsidianServer can be constructed from the library crate.
// This proves the server module is properly re-exported.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn server_construction_and_get_info() {
    let (_mock, server) = setup().await;
    let info = server.get_info();
    assert_eq!(info.server_info.name, "obsidian-mcp");
    assert!(info.capabilities.tools.is_some());
}
