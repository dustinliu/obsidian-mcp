use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;

use obsidian_mcp::client::ObsidianClient;
use obsidian_mcp::server::ObsidianServer;
use rmcp::model::{CallToolRequestParam, CallToolResult, RawContent};
use rmcp::service::RunningService;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::tower::{
    StreamableHttpServerConfig, StreamableHttpService,
};
use rmcp::{RoleClient, ServiceExt};
use serde_json::json;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{body_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Spin up wiremock + MCP server + MCP client, returning all three handles.
async fn setup() -> (
    MockServer,
    RunningService<RoleClient, ()>,
    CancellationToken,
) {
    // 1. Start wiremock (stands in for Obsidian REST API)
    let mock = MockServer::start().await;
    let client = Arc::new(ObsidianClient::new(mock.uri(), "test-key".to_string()));

    // 2. Build MCP server via Axum
    let cancel_token = CancellationToken::new();
    let config = StreamableHttpServerConfig {
        stateful_mode: true,
        cancellation_token: cancel_token.clone(),
        ..Default::default()
    };

    let session_manager = Arc::new(LocalSessionManager::default());
    let service = StreamableHttpService::new(
        move || Ok(ObsidianServer::new(client.clone())),
        session_manager,
        config,
    );

    let app = axum::Router::new().nest_service("/mcp", service);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 3. Create MCP client
    let mcp_url = format!("http://{}/mcp", addr);
    let transport = StreamableHttpClientTransport::from_uri(mcp_url);
    let mcp_client = ().serve(transport).await.unwrap();

    (mock, mcp_client, cancel_token)
}

/// Helper: call a tool by name with JSON arguments.
async fn call_tool(
    client: &RunningService<RoleClient, ()>,
    name: &str,
    args: serde_json::Value,
) -> CallToolResult {
    let arguments = match args {
        serde_json::Value::Object(map) => Some(map),
        _ => None,
    };
    client
        .peer()
        .call_tool(CallToolRequestParam {
            name: Cow::Owned(name.to_string()),
            arguments,
        })
        .await
        .unwrap()
}

/// Extract the first text content from a CallToolResult.
fn first_text(result: &CallToolResult) -> &str {
    result
        .content
        .iter()
        .find_map(|c| match &c.raw {
            RawContent::Text(t) => Some(t.text.as_str()),
            _ => None,
        })
        .expect("expected text content in tool result")
}

// ---------------------------------------------------------------------------
// E2E Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_list_tools() {
    let (_mock, client, _cancel) = setup().await;

    let tools = client.peer().list_tools(Default::default()).await.unwrap();

    let names: Vec<&str> = tools.tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(tools.tools.len(), 16, "expected 16 tools, got: {:?}", names);

    // Spot-check a few tool names
    assert!(names.contains(&"read_note"), "missing read_note");
    assert!(names.contains(&"create_note"), "missing create_note");
    assert!(names.contains(&"search"), "missing search");
    assert!(names.contains(&"server_info"), "missing server_info");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_read_note() {
    let (mock, client, _cancel) = setup().await;

    Mock::given(method("GET"))
        .and(path("/vault/my/note.md"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Accept", "text/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_string("# My Note\n\nHello from e2e"))
        .mount(&mock)
        .await;

    let result = call_tool(&client, "read_note", json!({"path": "my/note.md"})).await;
    assert_eq!(first_text(&result), "# My Note\n\nHello from e2e");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_create_note() {
    let (mock, client, _cancel) = setup().await;

    Mock::given(method("PUT"))
        .and(path("/vault/new/doc.md"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "text/markdown"))
        .and(body_string("# Fresh"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let result = call_tool(
        &client,
        "create_note",
        json!({"path": "new/doc.md", "content": "# Fresh"}),
    )
    .await;
    assert!(first_text(&result).contains("Created note at new/doc.md"));

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_search() {
    let (mock, client, _cancel) = setup().await;

    Mock::given(method("POST"))
        .and(path("/search/simple/"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "text/plain"))
        .and(body_string("rust programming"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!([{"filename": "rust.md", "score": 0.95}])),
        )
        .mount(&mock)
        .await;

    let result = call_tool(&client, "search", json!({"query": "rust programming"})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(parsed[0]["filename"], "rust.md");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_list_files() {
    let (mock, client, _cancel) = setup().await;

    Mock::given(method("GET"))
        .and(path("/vault/"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Accept", "application/json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"files": ["a.md", "b.md", "sub/c.md"]})),
        )
        .mount(&mock)
        .await;

    let result = call_tool(&client, "list_files", json!({})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    let files = parsed["files"].as_array().unwrap();
    assert_eq!(files.len(), 3);

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_delete_note() {
    let (mock, client, _cancel) = setup().await;

    Mock::given(method("DELETE"))
        .and(path("/vault/old-note.md"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock)
        .await;

    let result = call_tool(&client, "delete_note", json!({"path": "old-note.md"})).await;
    assert!(first_text(&result).contains("Deleted old-note.md"));

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_server_info() {
    let (mock, client, _cancel) = setup().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "OK", "versions": {"self": "1.0"}})),
        )
        .mount(&mock)
        .await;

    let result = call_tool(&client, "server_info", json!({})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(parsed["status"], "OK");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_error_propagation() {
    let (mock, client, _cancel) = setup().await;

    Mock::given(method("GET"))
        .and(path("/vault/missing.md"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&mock)
        .await;

    let arguments = json!({"path": "missing.md"});
    let arguments = match arguments {
        serde_json::Value::Object(map) => Some(map),
        _ => None,
    };
    let err = client
        .peer()
        .call_tool(CallToolRequestParam {
            name: Cow::Owned("read_note".to_string()),
            arguments,
        })
        .await;

    // The server maps AppError to McpError, so call_tool returns Err
    assert!(err.is_err(), "expected MCP error for 404 response");
    let err_msg = format!("{}", err.unwrap_err());
    assert!(
        err_msg.contains("404"),
        "error should mention status 404, got: {}",
        err_msg
    );

    client.cancel().await.unwrap();
}
