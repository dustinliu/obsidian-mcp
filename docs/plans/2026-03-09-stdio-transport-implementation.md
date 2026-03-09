# stdio Transport Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add stdio as the default MCP transport, with HTTP as an opt-in alternative via `--transport` flag.

**Architecture:** Add a `Transport` enum to CLI args. Branch in `main()` based on the selected transport: stdio uses `rmcp::transport::io::stdio()` with `ServiceExt::serve`, HTTP keeps the existing Axum setup unchanged.

**Tech Stack:** rmcp (with `transport-io` feature), clap, tokio

---

### Task 1: Add `transport-io` feature to rmcp dependency

**Files:**
- Modify: `Cargo.toml:7-11`

**Step 1: Add the feature**

In `Cargo.toml`, add `"transport-io"` to the rmcp features list:

```toml
rmcp = { version = "0.12", features = [
    "server",
    "macros",
    "transport-streamable-http-server",
    "transport-io",
] }
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: success (no errors)

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "build: add rmcp transport-io feature for stdio support"
```

---

### Task 2: Add `--transport` CLI flag and `Transport` enum

**Files:**
- Modify: `src/main.rs:1-38` (imports and Cli struct)
- Test: `src/main.rs` (unit test module at end of file)

**Step 1: Write the failing test**

Add a `#[cfg(test)]` module at the end of `src/main.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_default_transport_is_stdio() {
        let cli = Cli::try_parse_from(["obsidian-mcp", "--api-key", "test123"]).unwrap();
        assert_eq!(cli.transport, Transport::Stdio);
    }

    #[test]
    fn test_transport_http() {
        let cli =
            Cli::try_parse_from(["obsidian-mcp", "--api-key", "test123", "--transport", "http"])
                .unwrap();
        assert_eq!(cli.transport, Transport::Http);
    }

    #[test]
    fn test_transport_stdio_explicit() {
        let cli =
            Cli::try_parse_from(["obsidian-mcp", "--api-key", "test123", "--transport", "stdio"])
                .unwrap();
        assert_eq!(cli.transport, Transport::Stdio);
    }

    #[test]
    fn test_invalid_transport_rejected() {
        let result =
            Cli::try_parse_from(["obsidian-mcp", "--api-key", "test123", "--transport", "grpc"]);
        assert!(result.is_err());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --bin obsidian-mcp -- tests -v`
Expected: FAIL — `Transport` type does not exist

**Step 3: Write the implementation**

Add the `Transport` enum and the `--transport` field to `Cli` in `src/main.rs`. Add `clap::ValueEnum` derive:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Transport {
    Stdio,
    Http,
}
```

Add to the `Cli` struct, before the `port` field:

```rust
    /// Transport mode
    #[arg(long, env = "MCP_TRANSPORT", default_value = "stdio")]
    transport: Transport,
```

**Step 4: Run test to verify it passes**

Run: `cargo test --bin obsidian-mcp -- tests -v`
Expected: all 4 tests PASS

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: add --transport CLI flag with stdio as default"
```

---

### Task 3: Implement stdio transport branch in main()

**Files:**
- Modify: `src/main.rs:40-96` (main function)

**Step 1: Write the failing test**

This task modifies the `main()` runtime path which is hard to unit-test directly. Instead, we write an integration-style test that verifies the server can start over stdio and respond to an `initialize` request via piped stdin/stdout.

Add a new test file `tests/test_stdio.rs`:

```rust
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

/// Send a JSON-RPC message with Content-Length header (as per MCP stdio protocol)
fn encode_jsonrpc(json: &str) -> Vec<u8> {
    format!("{json}\n").into_bytes()
}

#[tokio::test]
async fn test_stdio_transport_initialize() {
    // This test requires OBSIDIAN_API_KEY and a running Obsidian instance,
    // same as e2e tests.
    let _ = dotenvy::dotenv();
    let api_key = match std::env::var("OBSIDIAN_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("OBSIDIAN_API_KEY not set, skipping stdio test");
            return;
        }
    };

    let api_url =
        std::env::var("OBSIDIAN_API_URL").unwrap_or_else(|_| "https://127.0.0.1:27124".into());

    let mut child = Command::new(env!("CARGO_BIN_EXE_obsidian-mcp"))
        .arg("--api-key")
        .arg(&api_key)
        .arg("--api-url")
        .arg(&api_url)
        .arg("--transport")
        .arg("stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn obsidian-mcp");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    // Send initialize request
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "0.1.0"
            }
        }
    });

    let msg = encode_jsonrpc(&init_request.to_string());
    stdin.write_all(&msg).await.unwrap();
    stdin.flush().await.unwrap();

    // Read response (with timeout)
    let mut buf = vec![0u8; 4096];
    let n = timeout(Duration::from_secs(10), stdout.read(&mut buf))
        .await
        .expect("Timed out waiting for response")
        .expect("Failed to read stdout");

    let response = String::from_utf8_lossy(&buf[..n]);
    assert!(
        response.contains("\"serverInfo\""),
        "Expected initialize response with serverInfo, got: {response}"
    );

    // Clean up
    child.kill().await.ok();
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test test_stdio -- -v --nocapture`
Expected: FAIL — the binary doesn't accept `--transport stdio` yet in its runtime path (it will parse but always run HTTP)

Actually, after Task 2 the flag parses but `main()` doesn't use it. The test will fail because the process will try to bind an HTTP port instead of doing stdio.

**Step 3: Write the implementation**

Modify `main()` in `src/main.rs`. Add `use rmcp::ServiceExt;` to imports. Replace the MCP server setup section (lines 64-95) with a transport branch:

```rust
    match cli.transport {
        Transport::Stdio => {
            tracing::info!("Starting MCP server with stdio transport");
            let server = ObsidianServer::new(client);
            let service = server
                .serve(rmcp::transport::io::stdio())
                .await
                .inspect_err(|e| tracing::error!("Server error: {}", e))?;
            service.waiting().await?;
        }
        Transport::Http => {
            let cancel_token = CancellationToken::new();
            let config = StreamableHttpServerConfig {
                stateful_mode: true,
                cancellation_token: cancel_token.clone(),
                ..Default::default()
            };

            let session_manager = Arc::new(LocalSessionManager::default());
            let client_clone = client.clone();
            let service = StreamableHttpService::new(
                move || Ok(ObsidianServer::new(client_clone.clone())),
                session_manager,
                config,
            );

            let app = axum::Router::new().nest_service("/mcp", service);

            let addr: SocketAddr = format!("{}:{}", cli.host, cli.port).parse()?;
            tracing::info!("MCP server listening on {}", addr);

            let listener = tokio::net::TcpListener::bind(addr).await?;

            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    tokio::signal::ctrl_c().await.ok();
                    tracing::info!("Shutting down...");
                    cancel_token.cancel();
                })
                .await?;
        }
    }
```

Remove `tokio_util` from imports if it's only used for `CancellationToken` — actually it's still used in the HTTP branch, so move the import inside the match or keep it. Simplest: keep the import.

**Step 4: Run test to verify it passes**

Run: `cargo test --test test_stdio -- -v --nocapture`
Expected: PASS (with OBSIDIAN_API_KEY set)

Also run existing tests to verify no regression:

Run: `just unit-test`
Expected: all existing tests PASS

**Step 5: Run lint**

Run: `just lint`
Expected: PASS

**Step 6: Commit**

```bash
git add src/main.rs tests/test_stdio.rs
git commit -m "feat: implement stdio transport as default MCP transport mode"
```

---

### Task 4: Update documentation

**Files:**
- Modify: `CLAUDE.md`
- Modify: `README.md`

**Step 1: Update CLAUDE.md**

In the Architecture section, update the transport description:

> This is an MCP server that bridges AI assistants to Obsidian vaults via the Local REST API plugin. It supports two transport modes: **stdio** (default, for Claude Desktop and similar clients) and **Streamable HTTP** (opt-in via `--transport http`).

In the Build & Run section, update the `just run` example:

```bash
just run                 # Run with stdio transport (default)
just run -- --transport http  # Run with HTTP transport
```

**Step 2: Update README.md**

Add usage examples showing both transport modes. Update the configuration section to document `--transport` flag and `MCP_TRANSPORT` env var.

**Step 3: Commit**

```bash
git add CLAUDE.md README.md
git commit -m "docs: update documentation for stdio transport support"
```
