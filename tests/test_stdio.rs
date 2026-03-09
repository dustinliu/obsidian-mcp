use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::{Duration, timeout};

/// Encode a JSON-RPC message for stdio transport
fn encode_jsonrpc(json: &str) -> Vec<u8> {
    format!("{json}\n").into_bytes()
}

#[tokio::test]
async fn test_stdio_transport_initialize() {
    // This test requires OBSIDIAN_API_KEY and a running Obsidian instance
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
