# Add stdio Transport Support

## Summary

Add stdio transport as the default MCP transport mode alongside the existing Streamable HTTP transport. This enables compatibility with Claude Desktop and other stdio-based MCP clients.

## CLI Interface

```
obsidian-mcp --api-key <KEY> [--transport stdio|http] [--port 3000] [--host 127.0.0.1]
```

- `--transport` defaults to `stdio`
- `--port` and `--host` are only meaningful in HTTP mode; silently ignored in stdio mode
- `--api-url` and `--api-key` are shared across both modes

## Architecture Changes

### Cargo.toml

Add rmcp feature `transport-io` to enable stdio support.

### src/main.rs

- Add `--transport` CLI parameter as an enum (`stdio` | `http`), defaulting to `stdio`
- Startup flow unchanged: parse CLI -> create `ObsidianClient` -> verify Obsidian connection -> branch by transport:
  - **stdio**: Use `rmcp::transport::io::stdio()` to get `(stdin, stdout)`, call `ObsidianServer::new(client).serve((stdin, stdout)).await`, wait for service to complete
  - **http**: Keep existing Axum + StreamableHttpService logic unchanged

### No changes needed

- `src/server.rs` — `ObsidianServer` is transport-agnostic
- `src/client.rs` — HTTP client to Obsidian API, unrelated to MCP transport
- `src/types.rs` — shared types, unrelated
- `src/error.rs` — error types, unrelated

## Logging

`tracing_subscriber::fmt()` defaults to stderr, which is safe for both transport modes. Verify no `println!` calls exist that could pollute stdout in stdio mode.

## Testing

- Unit test: verify server starts and handles JSON-RPC messages over stdio transport (mock stdin/stdout)
- Existing HTTP e2e tests remain unchanged

## Decisions

- stdio is the default transport (for Claude Desktop compatibility)
- HTTP-only flags (`--port`, `--host`) are silently ignored in stdio mode
- Startup connection check (fail fast) is preserved for both transport modes
