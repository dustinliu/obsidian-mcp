use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::{Arc, Once};

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
use serial_test::serial;
use tokio_util::sync::CancellationToken;

const DEFAULT_OBSIDIAN_API_URL: &str = "https://127.0.0.1:27124";
const TEST_FOLDER: &str = "tests";

static INIT_DOTENV: Once = Once::new();

/// Load `.env` file once (no-op if the file doesn't exist).
fn init_dotenv() {
    INIT_DOTENV.call_once(|| {
        let _ = dotenvy::dotenv();
    });
}

fn obsidian_api_url() -> String {
    init_dotenv();
    std::env::var("OBSIDIAN_API_URL").unwrap_or_else(|_| DEFAULT_OBSIDIAN_API_URL.to_string())
}

/// Read the API key from the OBSIDIAN_API_KEY environment variable.
fn api_key() -> Option<String> {
    init_dotenv();
    std::env::var("OBSIDIAN_API_KEY").ok()
}

/// Macro that skips the test if OBSIDIAN_API_KEY is not set.
macro_rules! require_api_key {
    () => {
        match api_key() {
            Some(key) => key,
            None => {
                eprintln!("OBSIDIAN_API_KEY not set, skipping test");
                return;
            }
        }
    };
}

/// Spin up MCP server + MCP client connected to the real Obsidian API.
/// Returns the MCP client, cancellation token, and raw ObsidianClient (for cleanup).
async fn setup(
    api_key: &str,
) -> (
    RunningService<RoleClient, ()>,
    CancellationToken,
    Arc<ObsidianClient>,
) {
    let client = Arc::new(ObsidianClient::new(obsidian_api_url(), api_key.to_string()));

    let cancel_token = CancellationToken::new();
    let config = StreamableHttpServerConfig {
        stateful_mode: true,
        cancellation_token: cancel_token.clone(),
        ..Default::default()
    };

    let session_manager = Arc::new(LocalSessionManager::default());
    let client_for_server = client.clone();
    let service = StreamableHttpService::new(
        move || Ok(ObsidianServer::new(client_for_server.clone())),
        session_manager,
        config,
    );

    let app = axum::Router::new().nest_service("/mcp", service);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();

    let shutdown_token = cancel_token.clone();
    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move { shutdown_token.cancelled().await })
            .await
            .unwrap();
    });

    let mcp_url = format!("http://{}/mcp", addr);
    let transport = StreamableHttpClientTransport::from_uri(mcp_url);
    let mcp_client = ().serve(transport).await.unwrap();

    (mcp_client, cancel_token, client)
}

/// Delete all files inside the `tests/` folder in the vault.
async fn cleanup(raw_client: &ObsidianClient) {
    let result = raw_client.list_files(Some(TEST_FOLDER)).await;
    if let Ok(val) = result {
        if let Some(files) = val.get("files").and_then(|f| f.as_array()) {
            for file in files {
                if let Some(path) = file.as_str() {
                    let _ = raw_client.delete_note(path).await;
                }
            }
        }
    }
}

/// Helper: call a tool by name with JSON arguments, returning the raw Result.
async fn try_call_tool(
    client: &RunningService<RoleClient, ()>,
    name: &str,
    args: serde_json::Value,
) -> Result<CallToolResult, rmcp::service::ServiceError> {
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
}

/// Helper: call a tool by name with JSON arguments, panicking on error.
async fn call_tool(
    client: &RunningService<RoleClient, ()>,
    name: &str,
    args: serde_json::Value,
) -> CallToolResult {
    try_call_tool(client, name, args).await.unwrap()
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
// CRUD Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_list_tools() {
    let key = require_api_key!();
    let (client, cancel, _raw) = setup(&key).await;

    let tools = client.peer().list_tools(Default::default()).await.unwrap();

    let names: Vec<&str> = tools.tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(tools.tools.len(), 16, "expected 16 tools, got: {:?}", names);

    assert!(names.contains(&"read_note"), "missing read_note");
    assert!(names.contains(&"create_note"), "missing create_note");
    assert!(names.contains(&"search"), "missing search");
    assert!(names.contains(&"server_info"), "missing server_info");

    cancel.cancel();
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_create_and_read_note() {
    let key = require_api_key!();
    let (client, cancel, raw) = setup(&key).await;
    cleanup(&raw).await;

    let note_path = format!("{}/e2e-create-read.md", TEST_FOLDER);
    let content = "# E2E Test\n\nHello from integration test";

    // Create
    let result = call_tool(
        &client,
        "create_note",
        json!({"path": note_path, "content": content}),
    )
    .await;
    let text = first_text(&result);
    assert!(
        text.contains("Created note at"),
        "unexpected create result: {}",
        text
    );

    // Read back
    let result = call_tool(&client, "read_note", json!({"path": note_path})).await;
    assert_eq!(first_text(&result), content);

    cancel.cancel();
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_append_note() {
    let key = require_api_key!();
    let (client, cancel, raw) = setup(&key).await;
    cleanup(&raw).await;

    let note_path = format!("{}/e2e-append.md", TEST_FOLDER);

    // Create initial note
    call_tool(
        &client,
        "create_note",
        json!({"path": note_path, "content": "line1"}),
    )
    .await;

    // Append
    let result = call_tool(
        &client,
        "append_note",
        json!({"path": note_path, "content": "\nline2"}),
    )
    .await;
    let text = first_text(&result);
    assert!(
        text.contains("Appended to"),
        "unexpected append result: {}",
        text
    );

    // Read back and verify
    let result = call_tool(&client, "read_note", json!({"path": note_path})).await;
    let text = first_text(&result);
    assert!(text.contains("line1"), "missing line1 in: {}", text);
    assert!(text.contains("line2"), "missing line2 in: {}", text);

    cancel.cancel();
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_delete_note() {
    let key = require_api_key!();
    let (client, cancel, raw) = setup(&key).await;
    cleanup(&raw).await;

    let note_path = format!("{}/e2e-delete.md", TEST_FOLDER);

    // Create a note first
    call_tool(
        &client,
        "create_note",
        json!({"path": note_path, "content": "to be deleted"}),
    )
    .await;

    // Delete it
    let result = call_tool(&client, "delete_note", json!({"path": note_path})).await;
    let text = first_text(&result);
    assert!(
        text.contains("Deleted"),
        "unexpected delete result: {}",
        text
    );

    // Verify it's gone by trying to read (should error)
    let err = try_call_tool(&client, "read_note", json!({"path": note_path})).await;
    assert!(err.is_err(), "expected error reading deleted note");

    cancel.cancel();
    client.cancel().await.unwrap();
}

// ---------------------------------------------------------------------------
// Patch Note Test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_patch_note() {
    let key = require_api_key!();
    let (client, cancel, raw) = setup(&key).await;
    cleanup(&raw).await;

    let note_path = format!("{}/e2e-patch.md", TEST_FOLDER);
    let initial_content =
        "# Heading 1\n\nContent under heading 1\n\n# Heading 2\n\nContent under heading 2";

    // Create note with headings
    call_tool(
        &client,
        "create_note",
        json!({"path": note_path, "content": initial_content}),
    )
    .await;

    // Patch: append under Heading 1
    let patch_content = "\nPatched content here";
    let result = call_tool(
        &client,
        "patch_note",
        json!({
            "path": note_path,
            "operation": "append",
            "target_type": "heading",
            "target": "Heading 1",
            "content": patch_content
        }),
    )
    .await;
    let patch_text = first_text(&result);
    assert!(
        !patch_text.is_empty(),
        "patch_note should return non-empty response"
    );

    // Read back and verify patch was applied
    let result = call_tool(&client, "read_note", json!({"path": note_path})).await;
    let text = first_text(&result);
    assert!(
        text.contains("Patched content here"),
        "patched content not found in: {}",
        text
    );
    assert!(
        text.contains("Content under heading 2"),
        "heading 2 content missing in: {}",
        text
    );

    cancel.cancel();
    client.cancel().await.unwrap();
}

// ---------------------------------------------------------------------------
// Query Tool Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_list_files() {
    let key = require_api_key!();
    let (client, cancel, raw) = setup(&key).await;
    cleanup(&raw).await;

    // Create a note so there's something to list
    let note_path = format!("{}/e2e-list-files.md", TEST_FOLDER);
    call_tool(
        &client,
        "create_note",
        json!({"path": note_path, "content": "list test"}),
    )
    .await;

    // List files in the test folder
    let result = call_tool(&client, "list_files", json!({"path": TEST_FOLDER})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    let files = parsed["files"].as_array().expect("expected files array");
    assert!(
        files.iter().any(|f| f
            .as_str()
            .map_or(false, |s| s.contains("e2e-list-files.md"))),
        "expected to find e2e-list-files.md in: {:?}",
        files
    );

    cancel.cancel();
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_search() {
    let key = require_api_key!();
    let (client, cancel, raw) = setup(&key).await;
    cleanup(&raw).await;

    let result = call_tool(&client, "search", json!({"query": "test"})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert!(
        parsed.is_array(),
        "search result should be JSON array, got: {}",
        text
    );

    cancel.cancel();
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_search_query() {
    let key = require_api_key!();
    let (client, cancel, _raw) = setup(&key).await;

    // Dataview DQL query — just verify the API call succeeds
    let result = call_tool(
        &client,
        "search_query",
        json!({"query": "TABLE file.name FROM \"\""}),
    )
    .await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert!(
        parsed.is_array() || parsed.is_object(),
        "search_query result should be JSON array or object, got: {}",
        text
    );

    cancel.cancel();
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_server_info() {
    let key = require_api_key!();
    let (client, cancel, _raw) = setup(&key).await;

    let result = call_tool(&client, "server_info", json!({})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(parsed["status"], "OK", "server_info status should be OK");
    assert!(
        parsed.get("versions").is_some(),
        "server_info should include versions"
    );

    cancel.cancel();
    client.cancel().await.unwrap();
}

// ---------------------------------------------------------------------------
// UI Tool Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_list_commands() {
    let key = require_api_key!();
    let (client, cancel, _raw) = setup(&key).await;

    let result = call_tool(&client, "list_commands", json!({})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert!(
        parsed.get("commands").is_some(),
        "list_commands should return commands key, got: {}",
        text
    );

    cancel.cancel();
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_execute_command() {
    let key = require_api_key!();
    let (client, cancel, _raw) = setup(&key).await;

    // Execute a safe, read-only command
    let result = call_tool(
        &client,
        "execute_command",
        json!({"command_id": "app:go-back"}),
    )
    .await;
    let text = first_text(&result);
    assert!(
        text.contains("Executed command"),
        "unexpected execute_command result: {}",
        text
    );

    cancel.cancel();
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_open_file() {
    let key = require_api_key!();
    let (client, cancel, raw) = setup(&key).await;
    cleanup(&raw).await;

    // Create a note to open
    let note_path = format!("{}/e2e-open-file.md", TEST_FOLDER);
    call_tool(
        &client,
        "create_note",
        json!({"path": note_path, "content": "open me"}),
    )
    .await;

    let result = call_tool(&client, "open_file", json!({"path": note_path})).await;
    let text = first_text(&result);
    assert!(
        text.contains("Opened"),
        "unexpected open_file result: {}",
        text
    );

    cancel.cancel();
    client.cancel().await.unwrap();
}

// ---------------------------------------------------------------------------
// Periodic Note Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[serial(daily_note)]
async fn e2e_periodic_note_crud() {
    let key = require_api_key!();
    let (client, cancel, raw) = setup(&key).await;

    // Save original daily note content for restoration
    let original_content = raw.get_periodic_note("daily", None, None, None).await.ok();

    // Update the daily note with test content
    let test_content = "# E2E Test Daily\n\nThis is test content for periodic note CRUD";
    let result = call_tool(
        &client,
        "update_periodic_note",
        json!({"period": "daily", "content": test_content}),
    )
    .await;
    let text = first_text(&result);
    assert!(
        text.contains("Updated"),
        "unexpected update result: {}",
        text
    );

    // Read it back
    let result = call_tool(&client, "get_periodic_note", json!({"period": "daily"})).await;
    assert_eq!(first_text(&result), test_content);

    // Append to it
    let append_content = "\n\nAppended periodic content";
    let result = call_tool(
        &client,
        "append_periodic_note",
        json!({"period": "daily", "content": append_content}),
    )
    .await;
    let text = first_text(&result);
    assert!(
        text.contains("Appended to"),
        "unexpected append result: {}",
        text
    );

    // Read back and verify append
    let result = call_tool(&client, "get_periodic_note", json!({"period": "daily"})).await;
    let text = first_text(&result);
    assert!(
        text.contains("Appended periodic content"),
        "appended content not found in: {}",
        text
    );

    // Restore original content
    if let Some(original) = original_content {
        raw.update_periodic_note("daily", None, None, None, &original)
            .await
            .ok();
    }

    cancel.cancel();
    client.cancel().await.unwrap();
}

#[tokio::test]
#[serial(daily_note)]
async fn e2e_patch_periodic_note() {
    let key = require_api_key!();
    let (client, cancel, raw) = setup(&key).await;

    // Save original daily note content for restoration
    let original_content = raw.get_periodic_note("daily", None, None, None).await.ok();

    // Set up a daily note with a top-level heading for patch targeting
    let initial = "# Tasks\n\n- existing task\n\n# Journal\n\nSome thoughts";
    raw.update_periodic_note("daily", None, None, None, initial)
        .await
        .unwrap();

    // Patch: append under Tasks heading
    let result = call_tool(
        &client,
        "patch_periodic_note",
        json!({
            "period": "daily",
            "operation": "append",
            "target_type": "heading",
            "target": "Tasks",
            "content": "\n- patched task from e2e"
        }),
    )
    .await;
    let patch_text = first_text(&result);
    assert!(
        !patch_text.is_empty(),
        "patch_periodic_note should return non-empty response"
    );

    // Read back and verify
    let result = call_tool(&client, "get_periodic_note", json!({"period": "daily"})).await;
    let text = first_text(&result);
    assert!(
        text.contains("patched task from e2e"),
        "patched content not found in: {}",
        text
    );
    assert!(
        text.contains("Some thoughts"),
        "journal content should still be present in: {}",
        text
    );

    // Restore original content
    if let Some(original) = original_content {
        raw.update_periodic_note("daily", None, None, None, &original)
            .await
            .ok();
    }

    cancel.cancel();
    client.cancel().await.unwrap();
}

// ---------------------------------------------------------------------------
// Error Propagation Test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_error_propagation() {
    let key = require_api_key!();
    let (client, cancel, _raw) = setup(&key).await;

    let err = try_call_tool(
        &client,
        "read_note",
        json!({"path": "nonexistent/does-not-exist-e2e-test.md"}),
    )
    .await;

    assert!(err.is_err(), "expected MCP error for nonexistent note");
    let err_msg = format!("{}", err.unwrap_err());
    assert!(
        err_msg.contains("404"),
        "error should mention status 404, got: {}",
        err_msg
    );

    cancel.cancel();
    client.cancel().await.unwrap();
}
