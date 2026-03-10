# Architecture

**Analysis Date:** 2026-03-10

## Pattern Overview

**Overall:** MCP (Model Context Protocol) server with layered client-server architecture

**Key Characteristics:**
- Two-layer architecture: MCP Protocol Handler layer + HTTP Client layer
- Dual transport support: stdio (default) and HTTP
- Synchronous-looking async/await pattern backed by Tokio runtime
- Tool-based RPC model where each MCP tool maps to a specific Obsidian REST API operation
- Bearer token authentication with self-signed TLS certificate tolerance

## Layers

**Transport Layer:**
- Purpose: Handle MCP protocol framing and bi-directional communication
- Location: `src/main.rs` (lines 76-116)
- Contains: Stdio transport setup, HTTP server setup (Axum), MCP session management
- Depends on: `rmcp` crate (transport modules), Axum, Tokio
- Used by: Entry point; provides service connectivity for tool invocations

**Server Handler Layer:**
- Purpose: Implement MCP ServerHandler interface and define 16 MCP tools
- Location: `src/server.rs`
- Contains: `ObsidianServer` struct with `#[tool_router]` macro-generated handler methods
- Depends on: `ObsidianClient`, `rmcp::ServerHandler`, `rmcp::handler::server::tool::ToolRouter`
- Used by: Transport layer to handle incoming tool calls

**HTTP Client Layer:**
- Purpose: Wrap HTTP communication with Obsidian Local REST API
- Location: `src/client.rs`
- Contains: `ObsidianClient` struct with methods for vault operations (read, write, patch, search, etc.)
- Depends on: `reqwest` (HTTP client), AppError, PatchParams/types
- Used by: Server handler; isolated from transport concerns

**Type Layer:**
- Purpose: Define shared types for serialization/deserialization and JSON schema generation
- Location: `src/types.rs` (Operation, TargetType, PatchParams), `src/error.rs` (AppError)
- Contains: Enums with Deserialize+JsonSchema traits for tool argument validation
- Depends on: `serde`, `schemars`, `thiserror`
- Used by: Server handler (for tool args) and client (for API requests)

## Data Flow

**Tool Invocation Flow:**

1. Client sends MCP CallTool request (e.g., `read_note` with path argument)
2. Transport layer routes to `rmcp::ServerHandler::call_tool()`
3. `#[tool_router]` macro dispatches to corresponding method in `ObsidianServer` (e.g., `read_note()`)
4. Method deserializes `Parameters<ReadNoteArgs>` wrapper and validates against JsonSchema
5. Method calls corresponding `ObsidianClient` async method (e.g., `client.read_note()`)
6. `ObsidianClient.read_note()` constructs HTTP GET request with auth header to `/vault/{path}`
7. Response checked via `check_response()` (validates HTTP status); text extracted
8. Result wrapped in `CallToolResult::success()` and returned as MCP Content
9. Transport serializes response and sends back to client

**Patch Operation Flow (Complex Example):**

1. Client sends `patch_note` tool with PatchNoteArgs (operation, target_type, target, content, etc.)
2. Server method converts args to `PatchParams` struct
3. Client constructs PATCH request with:
   - URL: `/vault/{path}`
   - Headers: Operation, Target-Type, Target, Target-Delimiter (optional), Trim-Target-Whitespace, Create-Target-If-Missing, Content-Type
   - Body: Processed via `prepare_patch_body()` (adds `\n` for append operations)
4. Response returns updated note content
5. Server returns content in MCP result

**State Management:**

- `ObsidianClient` maintains connection pool and bearer token
- `ObsidianServer` holds Arc-wrapped client reference (shared across tool invocations)
- No application state; all state is in Obsidian vault
- Session state managed by transport (per-connection in HTTP mode, single session in stdio mode)

## Key Abstractions

**ObsidianClient:**
- Purpose: Abstract HTTP transport to Obsidian REST API
- Examples: `src/client.rs` (lines 14-334)
- Pattern: Builder constructor with self-signed cert tolerance, pre-formatted bearer token, fluent request building
- Key methods: `read_note()`, `create_note()`, `patch_note()`, `delete_note()`, `list_files()`, `search_simple()`, `search_query()`, `list_commands()`, `execute_command()`, `open_file()`, periodic note methods

**ObsidianServer:**
- Purpose: MCP protocol handler implementing ServerHandler with tool definitions
- Examples: `src/server.rs` (lines 17-420)
- Pattern: `#[tool_router]` macro generates tool dispatch and JSON schema exposure
- Each tool = a `#[tool]` attributed async method that deserializes args and calls client

**AppError:**
- Purpose: Unified error representation
- Examples: `src/error.rs` (lines 3-13)
- Pattern: `thiserror` enum with From implementations for `reqwest::Error` and `serde_json::Error`
- Variants: `Http`, `Api` (with status + body), `Json`

**Operation / TargetType / PatchParams:**
- Purpose: Strongly typed patch API parameters for v3 PATCH endpoint
- Examples: `src/types.rs` (lines 1-55)
- Pattern: Newtype enums for append/prepend/replace and heading/block/frontmatter; struct with optional fields
- Deserialization: lowercase enum variants (serde rename_all = "lowercase")

## Entry Points

**stdio Mode (Default):**
- Location: `src/main.rs` (lines 77-84)
- Triggers: `just run` or binary invocation with `--transport stdio`
- Responsibilities:
  - Parse CLI args (OBSIDIAN_API_URL, OBSIDIAN_API_KEY, transport, port, host)
  - Create ObsidianClient with API URL and key
  - Verify connection via `server_info()` call
  - Attach ObsidianServer to stdio transport and block waiting for messages

**HTTP Mode:**
- Location: `src/main.rs` (lines 86-116)
- Triggers: `--transport http --host 127.0.0.1 --port 3000`
- Responsibilities:
  - Parse CLI args
  - Create ObsidianClient, verify connection
  - Build StreamableHttpService with ObsidianServer factory
  - Mount at `/mcp` path in Axum router
  - Bind TCP listener and serve with graceful shutdown on Ctrl+C

## Error Handling

**Strategy:** Explicit error propagation with contextual information

**Patterns:**

- **HTTP errors:** `check_response()` in client converts non-success status to AppError::Api (with status code + response body)
- **Client errors:** `reqwest::Error` auto-converted to AppError::Http via From impl
- **JSON errors:** Serialization/deserialization failures become AppError::Json
- **MCP layer:** AppError converted to McpError via `to_mcp_error()` helper (internal_error variant with stringified message)
- **Graceful degradation:** Tool invocations that fail return McpError; connection stays open for next tool

## Cross-Cutting Concerns

**Logging:**
- Framework: `tracing` crate with `tracing_subscriber`
- Configuration: `RUST_LOG` env var; default minimum level "obsidian_mcp=info" set in main
- Output: stderr (configured in main.rs lines 53-59)
- Patterns: Connection verification logged at info level, errors logged at error level

**Validation:**
- Args validated by serde deserialize against JsonSchema during tool dispatch
- Patch query validation: `search_query()` tool enforces TABLE-only queries (rejects LIST/TASK at application level)
- HTTP responses validated via `check_response()` status check

**Authentication:**
- Bearer token strategy: `Authorization: Bearer {api_key}` header injected by ObsidianClient constructor
- Token pre-formatted and stored in client constructor; reused for all requests
- TLS cert validation disabled for local Obsidian API (self-signed certs via `danger_accept_invalid_certs()`)

---

*Architecture analysis: 2026-03-10*
