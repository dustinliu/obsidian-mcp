# Obsidian MCP Server — Design Document

**Date:** 2026-03-06

## Overview

An MCP (Model Context Protocol) server written in Rust that exposes Obsidian vault operations as tools for AI assistants. It communicates with Obsidian through the [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) plugin.

## Decisions

| Item | Decision |
|------|----------|
| Language | Rust |
| MCP SDK | `rmcp` (official Rust MCP SDK) |
| Transport | Streamable HTTP (stdio may be added later) |
| Backend | Obsidian Local REST API only |
| HTTP client | `reqwest` |
| CLI parser | `clap` |
| Config priority | CLI args > environment variables > defaults |
| SSL | Skip cert verification (`danger_accept_invalid_certs`) for self-signed certs |

## Architecture

```
┌───────────────────────────────────────────┐
│            obsidian-mcp binary            │
│                                           │
│  ┌─────────┐   ┌──────────────────────┐   │
│  │  CLI    │   │    MCP Server        │   │
│  │  Config │──>│    (rmcp)            │   │
│  │  Parser │   │                      │   │
│  └─────────┘   │  ┌────────────────┐  │   │
│                │  │  Tool Handlers │  │   │
│                │  │  (16 tools)    │  │   │
│                │  └───────┬────────┘  │   │
│                │          │           │   │
│                │  ┌───────v────────┐  │   │
│                │  │ ObsidianClient │  │   │
│                │  │  (reqwest)     │  │   │
│                │  └───────┬────────┘  │   │
│                │          │           │   │
│                └──────────┼───────────┘   │
│                           │ HTTPS         │
└───────────────────────────┼───────────────┘
                            v
                 ┌─────────────────────┐
                 │ Obsidian Local      │
                 │ REST API            │
                 │ (localhost:27124)   │
                 └─────────────────────┘
```

Three layers:

1. **CLI Config Parser** — `clap` parses startup arguments, falls back to environment variables.
2. **MCP Server** — `rmcp` provides Streamable HTTP transport, registers 16 tools, handles MCP protocol.
3. **ObsidianClient** — Wraps all HTTP calls to Local REST API via `reqwest`. Shared by all tool handlers.

## CLI Parameters

```
obsidian-mcp [OPTIONS]

Options:
    --api-url <URL>      Obsidian REST API URL [env: OBSIDIAN_API_URL] [default: https://127.0.0.1:27124]
    --api-key <KEY>      Obsidian REST API key [env: OBSIDIAN_API_KEY] (required)
    --port <PORT>        MCP server listen port [env: MCP_PORT] [default: 3000]
    --host <HOST>        MCP server listen host [env: MCP_HOST] [default: 127.0.0.1]
```

`api-key` is the only required parameter. The server exits with an error if not provided.

## Startup Flow

1. Parse CLI args + environment variables
2. Create `ObsidianClient` (reqwest + base_url + api_key)
3. Call `server_info()` to verify connection
   - Success → continue
   - Failure → print error and exit
4. Create `ObsidianServer` (with tool_router)
5. Start rmcp Streamable HTTP server on host:port
6. Wait for shutdown signal (Ctrl+C)

## ObsidianClient

```rust
struct ObsidianClient {
    http: reqwest::Client,  // configured with danger_accept_invalid_certs(true)
    base_url: String,
    api_key: String,
}
```

Responsibilities:
- One method per REST API endpoint
- All requests include `Authorization: Bearer <api_key>` header
- Accepts self-signed certificates
- Converts HTTP errors into a unified app error type

## Tools (16 total)

### Vault Files

| Tool | API Endpoint | Parameters |
|------|-------------|------------|
| `read_note` | `GET /vault/{filename}` | `path: String` |
| `create_note` | `PUT /vault/{filename}` | `path: String`, `content: String` |
| `append_note` | `POST /vault/{filename}` | `path: String`, `content: String` |
| `patch_note` | `PATCH /vault/{filename}` | `path: String`, `heading: Option<String>`, `content: String` |
| `delete_note` | `DELETE /vault/{filename}` | `path: String` |
| `list_files` | `GET /vault/{dir}` | `path: Option<String>` |

### Search

| Tool | API Endpoint | Parameters |
|------|-------------|------------|
| `search` | `POST /search/simple/` | `query: String` |
| `search_query` | `POST /search/` | `query: String` |

### Commands

| Tool | API Endpoint | Parameters |
|------|-------------|------------|
| `list_commands` | `GET /commands/` | (none) |
| `execute_command` | `POST /commands/{commandId}/` | `command_id: String` |

### Open

| Tool | API Endpoint | Parameters |
|------|-------------|------------|
| `open_file` | `POST /open/{filename}` | `path: String` |

### Periodic Notes

| Tool | API Endpoint | Parameters |
|------|-------------|------------|
| `get_periodic_note` | `GET /periodic/{period}/[{y}/{m}/{d}/]` | `period: String`, `year: Option<u32>`, `month: Option<u32>`, `day: Option<u32>` |
| `update_periodic_note` | `PUT /periodic/{period}/[{y}/{m}/{d}/]` | `period: String`, `year/month/day: Option`, `content: String` |
| `append_periodic_note` | `POST /periodic/{period}/[{y}/{m}/{d}/]` | `period: String`, `year/month/day: Option`, `content: String` |
| `patch_periodic_note` | `PATCH /periodic/{period}/[{y}/{m}/{d}/]` | `period: String`, `year/month/day: Option`, `heading: Option<String>`, `content: String` |

### System

| Tool | API Endpoint | Parameters |
|------|-------------|------------|
| `server_info` | `GET /` | (none) |

## Tool Handler Pattern

All tools follow the same pattern:

```rust
#[tool(description = "Read the content of a note at the given path")]
async fn read_note(
    &self,
    #[tool(param, description = "Path to the note, e.g. 'folder/note.md'")]
    path: String,
) -> Result<CallToolResult, McpError> {
    let content = self.client.read_note(&path).await?;
    Ok(CallToolResult::success(vec![Content::text(content)]))
}
```

Read operations return note content as text. List operations return JSON. Write operations return a success message.

## Project Structure

```
obsidian-mcp/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI parsing + startup flow
│   ├── client.rs            # ObsidianClient (reqwest wrapper)
│   ├── server.rs            # ObsidianServer + #[tool_router] 16 tools
│   └── error.rs             # Unified error type
├── README.md
├── LICENSE
└── .gitignore
```

## Key Dependencies

- `rmcp` — MCP protocol + Streamable HTTP server
- `reqwest` — HTTP client for Obsidian REST API
- `clap` — CLI argument parsing
- `tokio` — async runtime
- `serde` / `serde_json` — JSON serialization
- `schemars` — JSON Schema generation (required by rmcp tool params)
- `thiserror` — error type definition
