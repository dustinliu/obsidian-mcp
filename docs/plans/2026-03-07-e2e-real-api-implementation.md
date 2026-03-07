# E2E Tests Against Real Obsidian API — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace wiremock-based e2e tests with tests that run against the real Obsidian Local REST API.

**Architecture:** MCP client → Axum → ObsidianServer → ObsidianClient → real Obsidian API at `https://host.orb.internal:27124`. Tests skip gracefully when `OBSIDIAN_API_KEY` is not set.

**Tech Stack:** rmcp, axum, reqwest, tokio, serde_json (no new dependencies)

---

### Task 1: Create e2e testing prerequisites doc

**Files:**
- Create: `docs/e2e-testing.md`

**Step 1: Write the doc**

```markdown
# E2E Testing Prerequisites

The e2e integration tests run against a real Obsidian instance via the Local REST API.
They exercise the full MCP stack: MCP client → Axum HTTP → ObsidianServer → ObsidianClient → Obsidian REST API.

## Requirements

1. **Obsidian** running on the macOS host with a vault open
2. **Local REST API plugin** installed and enabled in Obsidian (default port: 27124)
3. **Network access** from the test environment to the Obsidian host
   - From OrbStack containers: `https://host.orb.internal:27124`

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `OBSIDIAN_API_KEY` | Yes | API key from Obsidian Local REST API plugin settings |

If `OBSIDIAN_API_KEY` is not set, all e2e tests are **skipped** (not failed).

## Running

```bash
# Set the API key first
export OBSIDIAN_API_KEY="your-api-key-here"

# Run e2e tests only
cargo make e2e

# Run all tests (unit + e2e)
cargo make test
```

## Test Isolation

- All write operations are scoped to the `tests/` folder inside the vault.
- Each test cleans up the `tests/` folder before and after execution.
- Periodic note tests modify the current period's note and restore it after.
- Tests run sequentially (`--test-threads=1`) to avoid race conditions on shared vault state.

## Troubleshooting

- **Tests skip silently:** Check that `OBSIDIAN_API_KEY` is exported in your shell.
- **Connection refused:** Ensure Obsidian is running and the Local REST API plugin is enabled.
- **TLS errors:** The test client accepts self-signed certificates (Obsidian uses self-signed TLS).
```

**Step 2: Commit**

```bash
git add docs/e2e-testing.md
git commit -m "docs: add e2e testing prerequisites guide"
```

---

### Task 2: Add e2e task to Makefile.toml

**Files:**
- Modify: `Makefile.toml:40-46`

**Step 1: Add e2e task**

Add after the existing `[tasks.test-verbose]` block:

```toml
[tasks.e2e]
command = "cargo"
args = ["test", "--test", "integration_test", "--", "--test-threads=1", "--nocapture"]
```

**Step 2: Verify the task is recognized**

Run: `cargo make --list-all-steps | grep e2e`
Expected: `e2e` appears in the list

**Step 3: Commit**

```bash
git add Makefile.toml
git commit -m "build: add cargo make e2e task for real API tests"
```

---

### Task 3: Rewrite integration test — setup and helpers

This task replaces the entire `tests/integration_test.rs` with a new version that connects to the real Obsidian API. Start with the setup/helper infrastructure only (no test functions yet).

**Files:**
- Modify: `tests/integration_test.rs` (full rewrite)

**Step 1: Write the setup and helper code**

Replace the entire file with:

```rust
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

const OBSIDIAN_API_URL: &str = "https://host.orb.internal:27124";
const TEST_FOLDER: &str = "tests";

/// Read the API key from the environment. Returns None if not set (tests will skip).
fn api_key() -> Option<String> {
    std::env::var("OBSIDIAN_API_KEY").ok()
}

/// Spin up MCP server + MCP client connected to the real Obsidian API.
async fn setup(api_key: &str) -> (RunningService<RoleClient, ()>, CancellationToken, Arc<ObsidianClient>) {
    let client = Arc::new(ObsidianClient::new(
        OBSIDIAN_API_URL.to_string(),
        api_key.to_string(),
    ));

    let cancel_token = CancellationToken::new();
    let config = StreamableHttpServerConfig {
        stateful_mode: true,
        cancellation_token: cancel_token.clone(),
        ..Default::default()
    };

    let session_manager = Arc::new(LocalSessionManager::default());
    let client_for_service = client.clone();
    let service = StreamableHttpService::new(
        move || Ok(ObsidianServer::new(client_for_service.clone())),
        session_manager,
        config,
    );

    let app = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let mcp_url = format!("http://{}/mcp", addr);
    let transport = StreamableHttpClientTransport::from_uri(mcp_url);
    let mcp_client = ().serve(transport).await.unwrap();

    (mcp_client, cancel_token, client)
}

/// Delete all notes in the tests/ folder using the ObsidianClient directly.
async fn cleanup(client: &ObsidianClient) {
    let result = client.list_files(Some(TEST_FOLDER)).await;
    if let Ok(value) = result {
        if let Some(files) = value["files"].as_array() {
            for file in files {
                if let Some(path) = file.as_str() {
                    let _ = client.delete_note(path).await;
                }
            }
        }
    }
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

/// Macro to skip a test when OBSIDIAN_API_KEY is not set.
macro_rules! require_api_key {
    () => {
        match api_key() {
            Some(key) => key,
            None => {
                eprintln!("OBSIDIAN_API_KEY not set — skipping e2e test");
                return;
            }
        }
    };
}
```

**Step 2: Verify it compiles**

Run: `cargo test --test integration_test --no-run`
Expected: compiles successfully (no test functions yet, that's fine — empty test binary)

**Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: rewrite e2e test setup for real Obsidian API"
```

---

### Task 4: Write CRUD note e2e tests

**Files:**
- Modify: `tests/integration_test.rs` (append test functions)

**Step 1: Add the CRUD tests**

Append to the file after the macro:

```rust
// ---------------------------------------------------------------------------
// E2E Tests — Tool listing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_list_tools() {
    let key = require_api_key!();
    let (mcp_client, _cancel, _client) = setup(&key).await;

    let tools = mcp_client.peer().list_tools(Default::default()).await.unwrap();
    let names: Vec<&str> = tools.tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(tools.tools.len(), 16, "expected 16 tools, got: {:?}", names);
    assert!(names.contains(&"read_note"), "missing read_note");
    assert!(names.contains(&"create_note"), "missing create_note");
    assert!(names.contains(&"search"), "missing search");
    assert!(names.contains(&"server_info"), "missing server_info");

    mcp_client.cancel().await.unwrap();
}

// ---------------------------------------------------------------------------
// E2E Tests — CRUD notes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_create_and_read_note() {
    let key = require_api_key!();
    let (mcp_client, _cancel, raw_client) = setup(&key).await;
    cleanup(&raw_client).await;

    // Create
    let result = call_tool(
        &mcp_client,
        "create_note",
        json!({"path": "tests/hello.md", "content": "# Hello\n\nWorld"}),
    )
    .await;
    assert!(first_text(&result).contains("Created note at tests/hello.md"));

    // Read back
    let result = call_tool(&mcp_client, "read_note", json!({"path": "tests/hello.md"})).await;
    assert_eq!(first_text(&result), "# Hello\n\nWorld");

    cleanup(&raw_client).await;
    mcp_client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_append_note() {
    let key = require_api_key!();
    let (mcp_client, _cancel, raw_client) = setup(&key).await;
    cleanup(&raw_client).await;

    // Create initial note
    call_tool(
        &mcp_client,
        "create_note",
        json!({"path": "tests/append.md", "content": "line1"}),
    )
    .await;

    // Append
    let result = call_tool(
        &mcp_client,
        "append_note",
        json!({"path": "tests/append.md", "content": "\nline2"}),
    )
    .await;
    assert!(first_text(&result).contains("Appended to tests/append.md"));

    // Verify
    let result = call_tool(&mcp_client, "read_note", json!({"path": "tests/append.md"})).await;
    let text = first_text(&result);
    assert!(text.contains("line1"), "missing line1 in: {}", text);
    assert!(text.contains("line2"), "missing line2 in: {}", text);

    cleanup(&raw_client).await;
    mcp_client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_delete_note() {
    let key = require_api_key!();
    let (mcp_client, _cancel, raw_client) = setup(&key).await;
    cleanup(&raw_client).await;

    // Create then delete
    call_tool(
        &mcp_client,
        "create_note",
        json!({"path": "tests/to-delete.md", "content": "temp"}),
    )
    .await;

    let result = call_tool(
        &mcp_client,
        "delete_note",
        json!({"path": "tests/to-delete.md"}),
    )
    .await;
    assert!(first_text(&result).contains("Deleted tests/to-delete.md"));

    // Verify it's gone — reading should fail
    let arguments = json!({"path": "tests/to-delete.md"});
    let arguments = match arguments {
        serde_json::Value::Object(map) => Some(map),
        _ => None,
    };
    let err = mcp_client
        .peer()
        .call_tool(CallToolRequestParam {
            name: Cow::Owned("read_note".to_string()),
            arguments,
        })
        .await;
    assert!(err.is_err(), "expected error reading deleted note");

    cleanup(&raw_client).await;
    mcp_client.cancel().await.unwrap();
}
```

**Step 2: Run and verify**

Run: `OBSIDIAN_API_KEY=<key> cargo test --test integration_test -- --test-threads=1 --nocapture`
Expected: all 4 tests pass

**Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add e2e CRUD note tests against real API"
```

---

### Task 5: Write patch_note e2e test

**Files:**
- Modify: `tests/integration_test.rs` (append)

**Step 1: Add patch_note test**

```rust
// ---------------------------------------------------------------------------
// E2E Tests — Patch note
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_patch_note() {
    let key = require_api_key!();
    let (mcp_client, _cancel, raw_client) = setup(&key).await;
    cleanup(&raw_client).await;

    // Create a note with a heading
    call_tool(
        &mcp_client,
        "create_note",
        json!({"path": "tests/patch.md", "content": "# Section 1\n\noriginal content\n\n# Section 2\n\nother content"}),
    )
    .await;

    // Patch: append under Section 1
    let result = call_tool(
        &mcp_client,
        "patch_note",
        json!({
            "path": "tests/patch.md",
            "operation": "append",
            "target_type": "heading",
            "target": "Section 1",
            "content": "\nappended under section 1"
        }),
    )
    .await;
    // patch_note returns the API response body text
    let _response = first_text(&result);

    // Read back and verify
    let result = call_tool(&mcp_client, "read_note", json!({"path": "tests/patch.md"})).await;
    let text = first_text(&result);
    assert!(
        text.contains("appended under section 1"),
        "patch content missing in: {}",
        text
    );
    assert!(
        text.contains("other content"),
        "Section 2 content should be preserved in: {}",
        text
    );

    cleanup(&raw_client).await;
    mcp_client.cancel().await.unwrap();
}
```

**Step 2: Run and verify**

Run: `OBSIDIAN_API_KEY=<key> cargo test --test integration_test e2e_patch_note -- --test-threads=1 --nocapture`
Expected: PASS

**Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add e2e patch_note test against real API"
```

---

### Task 6: Write list_files, search, search_query, server_info tests

**Files:**
- Modify: `tests/integration_test.rs` (append)

**Step 1: Add tests**

```rust
// ---------------------------------------------------------------------------
// E2E Tests — Query tools
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_list_files() {
    let key = require_api_key!();
    let (mcp_client, _cancel, raw_client) = setup(&key).await;
    cleanup(&raw_client).await;

    // Create a note so tests/ folder exists
    call_tool(
        &mcp_client,
        "create_note",
        json!({"path": "tests/listed.md", "content": "# Listed"}),
    )
    .await;

    // List root
    let result = call_tool(&mcp_client, "list_files", json!({})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert!(parsed["files"].is_array(), "expected files array");

    // List tests/ folder
    let result = call_tool(&mcp_client, "list_files", json!({"path": "tests"})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    let files = parsed["files"].as_array().unwrap();
    assert!(
        files.iter().any(|f| f.as_str().map_or(false, |s| s.contains("listed.md"))),
        "tests/listed.md should appear in file list: {:?}",
        files
    );

    cleanup(&raw_client).await;
    mcp_client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_search() {
    let key = require_api_key!();
    let (mcp_client, _cancel, raw_client) = setup(&key).await;
    cleanup(&raw_client).await;

    // Create a note with unique content
    call_tool(
        &mcp_client,
        "create_note",
        json!({"path": "tests/searchable.md", "content": "# Searchable\n\nzxyq9872 unique marker"}),
    )
    .await;

    // Give Obsidian a moment to index
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Search for the unique marker
    let result = call_tool(&mcp_client, "search", json!({"query": "zxyq9872"})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert!(parsed.is_array(), "search result should be an array");

    cleanup(&raw_client).await;
    mcp_client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_search_query() {
    let key = require_api_key!();
    let (mcp_client, _cancel, raw_client) = setup(&key).await;
    cleanup(&raw_client).await;

    // Run a simple Dataview query
    let result = call_tool(
        &mcp_client,
        "search_query",
        json!({"query": "LIST FROM \"tests\" LIMIT 5"}),
    )
    .await;
    let text = first_text(&result);
    // Just verify it returns valid JSON (Dataview must be installed)
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert!(
        parsed.is_object() || parsed.is_array(),
        "search_query should return JSON, got: {}",
        text
    );

    cleanup(&raw_client).await;
    mcp_client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_server_info() {
    let key = require_api_key!();
    let (mcp_client, _cancel, _raw_client) = setup(&key).await;

    let result = call_tool(&mcp_client, "server_info", json!({})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(parsed["status"], "OK", "server status should be OK");
    assert!(parsed["versions"].is_object(), "versions should be present");

    mcp_client.cancel().await.unwrap();
}
```

**Step 2: Run and verify**

Run: `OBSIDIAN_API_KEY=<key> cargo test --test integration_test -- --test-threads=1 --nocapture`
Expected: all pass

**Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add e2e list_files, search, search_query, server_info tests"
```

---

### Task 7: Write UI tool tests (list_commands, execute_command, open_file)

**Files:**
- Modify: `tests/integration_test.rs` (append)

**Step 1: Add UI tool tests**

These tests only verify the API call succeeds — no result assertion on side effects.

```rust
// ---------------------------------------------------------------------------
// E2E Tests — UI tools (verify call succeeds, no side-effect assertion)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_list_commands() {
    let key = require_api_key!();
    let (mcp_client, _cancel, _raw_client) = setup(&key).await;

    let result = call_tool(&mcp_client, "list_commands", json!({})).await;
    let text = first_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    // Just verify it returned valid JSON with commands
    assert!(
        parsed.is_object() || parsed.is_array(),
        "list_commands should return JSON"
    );

    mcp_client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_execute_command() {
    let key = require_api_key!();
    let (mcp_client, _cancel, _raw_client) = setup(&key).await;

    // Use a safe, idempotent command
    let result = call_tool(
        &mcp_client,
        "execute_command",
        json!({"command_id": "app:open-help"}),
    )
    .await;
    let text = first_text(&result);
    assert!(
        text.contains("Executed command"),
        "expected success message, got: {}",
        text
    );

    mcp_client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_open_file() {
    let key = require_api_key!();
    let (mcp_client, _cancel, raw_client) = setup(&key).await;
    cleanup(&raw_client).await;

    // Create a file first so we can open it
    call_tool(
        &mcp_client,
        "create_note",
        json!({"path": "tests/openme.md", "content": "# Open Me"}),
    )
    .await;

    let result = call_tool(
        &mcp_client,
        "open_file",
        json!({"path": "tests/openme.md"}),
    )
    .await;
    let text = first_text(&result);
    assert!(
        text.contains("Opened"),
        "expected success message, got: {}",
        text
    );

    cleanup(&raw_client).await;
    mcp_client.cancel().await.unwrap();
}
```

**Step 2: Run and verify**

Run: `OBSIDIAN_API_KEY=<key> cargo test --test integration_test e2e_list_commands e2e_execute_command e2e_open_file -- --test-threads=1 --nocapture`
Expected: all pass

**Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add e2e UI tool tests (list_commands, execute_command, open_file)"
```

---

### Task 8: Write periodic note e2e tests

**Files:**
- Modify: `tests/integration_test.rs` (append)

**Step 1: Add periodic note tests**

Periodic notes are managed by the Obsidian plugin and don't live under `tests/`. These tests use today's daily note. They save and restore original content.

```rust
// ---------------------------------------------------------------------------
// E2E Tests — Periodic notes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_periodic_note_crud() {
    let key = require_api_key!();
    let (mcp_client, _cancel, _raw_client) = setup(&key).await;

    // Save current daily note content (may not exist yet)
    let original = call_tool(
        &mcp_client,
        "get_periodic_note",
        json!({"period": "daily"}),
    )
    .await;
    let original_content = first_text(&original).to_string();

    // Update daily note with test content
    let result = call_tool(
        &mcp_client,
        "update_periodic_note",
        json!({"period": "daily", "content": "# E2E Test Daily\n\ntest content"}),
    )
    .await;
    assert!(
        first_text(&result).contains("Updated daily periodic note"),
        "expected update success message"
    );

    // Read it back
    let result = call_tool(
        &mcp_client,
        "get_periodic_note",
        json!({"period": "daily"}),
    )
    .await;
    assert_eq!(first_text(&result), "# E2E Test Daily\n\ntest content");

    // Append to it
    let result = call_tool(
        &mcp_client,
        "append_periodic_note",
        json!({"period": "daily", "content": "\nappended line"}),
    )
    .await;
    assert!(
        first_text(&result).contains("Appended to daily periodic note"),
        "expected append success message"
    );

    // Verify append
    let result = call_tool(
        &mcp_client,
        "get_periodic_note",
        json!({"period": "daily"}),
    )
    .await;
    let text = first_text(&result);
    assert!(text.contains("test content"), "original content missing");
    assert!(text.contains("appended line"), "appended content missing");

    // Restore original content
    call_tool(
        &mcp_client,
        "update_periodic_note",
        json!({"period": "daily", "content": original_content}),
    )
    .await;

    mcp_client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_patch_periodic_note() {
    let key = require_api_key!();
    let (mcp_client, _cancel, _raw_client) = setup(&key).await;

    // Save current daily note
    let original = call_tool(
        &mcp_client,
        "get_periodic_note",
        json!({"period": "daily"}),
    )
    .await;
    let original_content = first_text(&original).to_string();

    // Set up a note with a heading to patch
    call_tool(
        &mcp_client,
        "update_periodic_note",
        json!({"period": "daily", "content": "# Tasks\n\n- existing task\n\n# Notes\n\nsome notes"}),
    )
    .await;

    // Patch: append under Tasks heading
    let result = call_tool(
        &mcp_client,
        "patch_periodic_note",
        json!({
            "period": "daily",
            "operation": "append",
            "target_type": "heading",
            "target": "Tasks",
            "content": "\n- new task from e2e"
        }),
    )
    .await;
    let _response = first_text(&result);

    // Verify
    let result = call_tool(
        &mcp_client,
        "get_periodic_note",
        json!({"period": "daily"}),
    )
    .await;
    let text = first_text(&result);
    assert!(
        text.contains("new task from e2e"),
        "patched content missing in: {}",
        text
    );
    assert!(
        text.contains("some notes"),
        "Notes section should be preserved in: {}",
        text
    );

    // Restore original
    call_tool(
        &mcp_client,
        "update_periodic_note",
        json!({"period": "daily", "content": original_content}),
    )
    .await;

    mcp_client.cancel().await.unwrap();
}
```

**Step 2: Run and verify**

Run: `OBSIDIAN_API_KEY=<key> cargo test --test integration_test e2e_periodic -- --test-threads=1 --nocapture`
Expected: both pass

**Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add e2e periodic note tests against real API"
```

---

### Task 9: Write error propagation test

**Files:**
- Modify: `tests/integration_test.rs` (append)

**Step 1: Add error test**

```rust
// ---------------------------------------------------------------------------
// E2E Tests — Error propagation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_error_propagation() {
    let key = require_api_key!();
    let (mcp_client, _cancel, _raw_client) = setup(&key).await;

    // Try to read a note that doesn't exist
    let arguments = json!({"path": "tests/nonexistent-e2e-note-12345.md"});
    let arguments = match arguments {
        serde_json::Value::Object(map) => Some(map),
        _ => None,
    };
    let err = mcp_client
        .peer()
        .call_tool(CallToolRequestParam {
            name: Cow::Owned("read_note".to_string()),
            arguments,
        })
        .await;

    assert!(err.is_err(), "expected MCP error for nonexistent note");
    let err_msg = format!("{}", err.unwrap_err());
    assert!(
        err_msg.contains("404"),
        "error should mention 404, got: {}",
        err_msg
    );

    mcp_client.cancel().await.unwrap();
}
```

**Step 2: Run and verify**

Run: `OBSIDIAN_API_KEY=<key> cargo test --test integration_test e2e_error -- --test-threads=1 --nocapture`
Expected: PASS

**Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add e2e error propagation test"
```

---

### Task 10: Run full e2e suite and update CLAUDE.md

**Step 1: Run all e2e tests**

Run: `OBSIDIAN_API_KEY=<key> cargo make e2e`
Expected: all tests pass

**Step 2: Run unit tests to confirm they still work**

Run: `cargo test --lib`
Expected: all unit tests pass (client.rs and other unit tests still use wiremock)

**Step 3: Run clippy**

Run: `cargo make clippy`
Expected: no warnings

**Step 4: Update CLAUDE.md**

In the `Build & Run` section of `CLAUDE.md`, add a line after `cargo make test`:

```
cargo make e2e                 # Run e2e tests (requires OBSIDIAN_API_KEY, see docs/e2e-testing.md)
```

Update the test description paragraph to:

```
Unit tests in `src/client.rs` and `src/server.rs` use wiremock to mock the Obsidian REST API. E2e tests in `tests/integration_test.rs` run against the real Obsidian Local REST API (see `docs/e2e-testing.md` for prerequisites).
```

**Step 5: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with e2e test instructions"
```
