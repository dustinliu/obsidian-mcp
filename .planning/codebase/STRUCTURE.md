# Codebase Structure

**Analysis Date:** 2026-03-10

## Directory Layout

```
obsidian-mcp/
├── src/                    # Rust source code
│   ├── main.rs            # CLI entry point and transport setup (stdio vs HTTP)
│   ├── lib.rs             # Re-exports of modules as public API
│   ├── server.rs          # MCP ServerHandler and 16 tool definitions
│   ├── client.rs          # ObsidianClient wrapper around reqwest HTTP client
│   ├── types.rs           # Shared types (Operation, TargetType, PatchParams)
│   └── error.rs           # AppError enum
├── tests/                 # Integration and transport tests
│   ├── integration_test.rs # E2e tests against real Obsidian REST API
│   └── test_stdio.rs      # Stdio transport smoke tests
├── docs/                  # Documentation
│   ├── e2e-testing.md    # Prerequisites and setup for e2e tests
│   └── ...               # Other docs
├── Cargo.toml            # Rust dependencies and project metadata
├── Cargo.lock            # Dependency lock file
├── justfile              # Task orchestration (build, test, fmt, lint, etc.)
├── CLAUDE.md             # Project instructions and design decisions
└── README.md             # User-facing documentation
```

## Directory Purposes

**src/ — Application Source:**
- Purpose: Core MCP server implementation
- Contains: Entry point, protocol handler, HTTP client, type definitions, error handling
- Key files: `server.rs` (largest: 1026 lines), `client.rs` (1026 lines), others <100 lines each

**tests/ — Automated Tests:**
- Purpose: Verify tool behavior and transport correctness
- Contains: Unit tests (in respective src files via #[cfg(test)]), integration tests (HTTP client + real API)
- Key files:
  - `integration_test.rs`: End-to-end tests requiring OBSIDIAN_API_KEY environment variable
  - `test_stdio.rs`: Smoke tests for stdio transport

**docs/ — Reference Documentation:**
- Purpose: User guides and setup instructions
- Key files:
  - `e2e-testing.md`: How to run integration tests (prerequisites: Obsidian vault, Local REST API plugin, API key)

## Key File Locations

**Entry Points:**
- `src/main.rs`: CLI parsing (clap) and transport mode selection (stdio vs HTTP)
  - Spawns tokio runtime, verifies Obsidian connection, initializes tracing
  - Default: stdio transport listening on stdin/stdout
  - Optional: HTTP transport binding to 127.0.0.1:3000 (configurable via CLI args/env vars)

**Configuration:**
- Environment variables consumed by CLI (via clap):
  - `OBSIDIAN_API_URL`: Obsidian REST API endpoint (default: https://127.0.0.1:27124)
  - `OBSIDIAN_API_KEY`: Bearer token for authentication (required)
  - `MCP_TRANSPORT`: stdio or http (default: stdio)
  - `MCP_PORT`: HTTP listen port (default: 3000)
  - `MCP_HOST`: HTTP listen host (default: 127.0.0.1)

**Core Logic:**
- `src/server.rs`: 16 MCP tools exposed via `#[tool_router]` macro
  - Note operations: `read_note`, `create_note`, `append_note`, `patch_note`, `delete_note`
  - Periodic note operations: `get_periodic_note`, `update_periodic_note`, `append_periodic_note`, `patch_periodic_note`
  - Vault operations: `list_files`, `search`, `search_query`
  - Command operations: `list_commands`, `execute_command`, `open_file`
- `src/client.rs`: HTTP interface to Obsidian Local REST API
  - HTTP methods mapped to operations: GET=read, PUT=create, POST=append, PATCH=patch, DELETE=delete
  - Request handling: URL construction, header injection (Authorization, Content-Type, Operation, Target-Type, etc.)
  - Response handling: Status validation via `check_response()`, text/JSON extraction
  - Special helpers: `periodic_url()` for date-based endpoints, `prepare_patch_body()` for newline normalization

**Testing:**
- `src/server.rs`: Unit tests for tools (wiremock-mocked Obsidian API) starting at line 122
- `src/client.rs`: Unit tests for HTTP layer (line 336+); 40+ tests covering CRUD operations, patch operations, periodic notes
- `tests/integration_test.rs`: E2e tests against real Obsidian vault (requires env var OBSIDIAN_API_KEY)

**Type Definitions:**
- `src/types.rs`:
  - `Operation` enum (append, prepend, replace)
  - `TargetType` enum (heading, block, frontmatter)
  - `PatchParams` struct (operation, target_type, target, optional: delimiter, trim, create, content_type)
- `src/error.rs`:
  - `AppError` enum (Http, Api, Json) with Display and From impls
  - Helper: `to_mcp_error()` conversion in server.rs

## Naming Conventions

**Files:**
- `src/[module].rs`: One module per file; modules re-exported in `lib.rs`
- `tests/test_*.rs`: Integration test suites
- `src/[module]/mod.rs`: Used for submodules (none currently; flat structure preferred)

**Functions:**
- `async fn [action]_[resource]()`: e.g., `read_note()`, `patch_note()`, `get_periodic_note()`
- `fn [helper]()`: Private helpers, e.g., `prepare_patch_body()`, `periodic_url()`, `check_response()`, `url()`, `to_mcp_error()`
- Tool methods in ObsidianServer: Match tool names exactly: `read_note`, `create_note`, `append_note`, etc.

**Variables:**
- CamelCase for types: `ObsidianClient`, `ObsidianServer`, `AppError`, `PatchParams`
- snake_case for values: `api_url`, `api_key`, `bearer_token`, `base_url`, `path`, `content`
- Abbreviations preserved: `http` (not capitalized), `mcp` (not capitalized)

**Types:**
- Public structs: `ObsidianClient`, `ObsidianServer`, `AppError`, `ServerInfo`, `PatchParams`
- Public enums: `Operation`, `TargetType` (with Display impl for string serialization)
- Arg structs (tool parameters): `ReadNoteArgs`, `CreateNoteArgs`, `PatchNoteArgs`, etc. (always suffixed with Args)

## Where to Add New Code

**New MCP Tool:**
1. Define arg struct in `src/server.rs` with `#[derive(Deserialize, JsonSchema)]`
2. Add `async fn [name](&self, Parameters(args): Parameters<[ArgsType]>) -> Result<CallToolResult, McpError>` in the `#[tool_router] impl ObsidianServer` block
3. Call corresponding `ObsidianClient` method or add new method to client if needed
4. Add unit tests in `src/server.rs` using wiremock to mock the Obsidian API endpoint
5. Add integration test in `tests/integration_test.rs` if it requires real API testing

**New Client Method (HTTP Operation):**
1. Add async method to `impl ObsidianClient` in `src/client.rs`
2. Follow pattern: construct URL via `self.url()` or `self.periodic_url()`, build request with headers, call `check_response()`, extract result
3. Add comprehensive unit tests using wiremock MockServer

**Shared Types:**
- Simple parameter structs: Add to tool arg structs in `src/server.rs`
- Reusable across multiple tools: Move to `src/types.rs` and add to public exports
- Error types: Add to `src/error.rs` with From implementations

**Utilities/Helpers:**
- HTTP request helpers: `src/client.rs` private methods
- Server response helpers: `src/server.rs` helpers like `to_mcp_error()`
- Type converters: `src/types.rs` with From/Into impls

## Special Directories

**target/ — Build Artifacts:**
- Purpose: Compiled binaries, test executables, documentation
- Generated: Yes (via `cargo build`, `cargo test`)
- Committed: No (.gitignore)

**.planning/ — GSD Codebase Documentation:**
- Purpose: Planning and analysis artifacts (not part of build)
- Generated: Yes (via `/gsd:map-codebase`)
- Committed: No (typically; check .gitignore)

**docs/ — User Documentation:**
- Purpose: Setup guides, API documentation, design decisions
- Generated: No (hand-written)
- Committed: Yes

**.env — Environment Configuration:**
- Purpose: Local environment variables for development
- Generated: No (created manually or via dotenvy)
- Committed: No (.gitignore; contains secrets)

---

*Structure analysis: 2026-03-10*
