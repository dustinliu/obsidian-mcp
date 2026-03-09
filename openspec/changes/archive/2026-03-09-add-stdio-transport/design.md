## Context

The server currently only supports Streamable HTTP transport via `mcp.run_streamable_http_async()`. Claude Desktop requires stdio transport, where the client spawns the server as a subprocess and communicates via stdin/stdout.

FastMCP (from the official `mcp` Python SDK) natively supports both transports — `run_stdio_async()` and `run_streamable_http_async()` — so no new dependencies are needed.

## Goals / Non-Goals

**Goals:**
- Support stdio transport for Claude Desktop compatibility
- Make stdio the default transport (most common MCP use case)
- Keep HTTP transport available via explicit `--transport http`

**Non-Goals:**
- SSE transport support (deprecated in MCP spec)
- Any changes to tool definitions, client, or server logic

## Decisions

### 1. CLI flag `--transport` with enum choices

Add `--transport` option with choices `stdio` and `http`, defaulting to `stdio`.

**Why not subcommands?** Both modes share identical options (`--api-url`, `--api-key`). A flag is simpler and avoids restructuring the CLI. `--host`/`--port` are harmlessly ignored in stdio mode.

### 2. Default to stdio (breaking change)

Stdio is the standard MCP transport and the primary use case (Claude Desktop). HTTP is the advanced deployment option.

**Migration**: Users running via `just deploy` or direct CLI with HTTP need to add `--transport http`.

### 3. Keep startup connectivity check for both transports

The `server_info()` check at startup provides fail-fast behavior regardless of transport. In stdio mode, the process exits with code 1, and Claude Desktop reports the failure.

## Risks / Trade-offs

- **[Breaking default]** Existing HTTP deployments break without `--transport http` → Mitigated by updating `just deploy` recipe and documenting in CLAUDE.md.
- **[Logging in stdio mode]** Python's `logging.basicConfig()` defaults to stderr, so no conflict with stdout used by stdio transport → No action needed.
