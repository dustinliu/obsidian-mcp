# External Integrations

**Analysis Date:** 2026-03-10

## APIs & External Services

**Obsidian Local REST API:**
- Service: Obsidian via Local REST API plugin (https://github.com/coddingtonbear/obsidian-local-rest-api)
- What it's used for: Vault operations (read/write/patch/delete notes, list files, search, execute commands)
- SDK/Client: `ObsidianClient` wrapper in `src/client.rs` using `reqwest::Client`
- Auth: Bearer token via `OBSIDIAN_API_KEY` environment variable
- Default URL: `https://127.0.0.1:27124`
- TLS: Accepts self-signed certificates (configured in `src/client.rs` via `danger_accept_invalid_certs(true)`)

## Data Storage

**Databases:**
- Not applicable - This is a stateless MCP server that reads/writes to Obsidian vault via REST API

**File Storage:**
- Obsidian Vault (accessed via REST API) - All note storage and retrieval happens through the Obsidian Local REST API
- No local file system storage

**Caching:**
- None - All requests are passthrough to Obsidian REST API

## Authentication & Identity

**Auth Provider:**
- Custom: Bearer token (pre-formatted string from `OBSIDIAN_API_KEY` environment variable)
- Implementation: In `src/client.rs`, token is formatted as `"Bearer {api_key}"` in the constructor and included in `Authorization` header for all requests

**Auth Endpoints:**
- No explicit OAuth/OIDC flow - simple bearer token authentication
- Token configured via environment variable at server startup

## Monitoring & Observability

**Error Tracking:**
- None - Errors are logged via tracing framework and converted to MCP error responses

**Logs:**
- Structured logging via `tracing` crate
- Configuration: `tracing-subscriber` with `env-filter` module
- Default level: `obsidian_mcp=info` (set in `src/main.rs` line 55-58)
- Control: Set `RUST_LOG` environment variable to adjust levels (e.g., `RUST_LOG=obsidian_mcp=debug`)
- Output: Directed to stderr (configured in `src/main.rs`)

**Observability Context:**
- Traces include:
  - Server startup and connection verification to Obsidian
  - API request details via `tracing::info!()` in `src/main.rs`
  - Errors from Obsidian API via `tracing::error!()`

## CI/CD & Deployment

**Hosting:**
- Not cloud-hosted - Runs locally on user's machine or as a service alongside Claude Desktop
- Transport options: stdio (default for Claude Desktop) or HTTP streamable (opt-in via `--transport http`)

**CI Pipeline:**
- Pre-release checks defined in `release.toml`: runs `just __check` (unit-test + lint + coverage + build)
- No external CI service configured; releases use `cargo-release` for version bumping and tagging

## Environment Configuration

**Required env vars:**
- `OBSIDIAN_API_KEY` - Bearer token for Obsidian Local REST API authentication (no default, must be provided)

**Optional env vars:**
- `OBSIDIAN_API_URL` - Base URL to Obsidian API (default: `https://127.0.0.1:27124`)
- `MCP_TRANSPORT` - Transport mode: "stdio" or "http" (default: "stdio")
- `MCP_PORT` - HTTP server port when using HTTP transport (default: "3000")
- `MCP_HOST` - HTTP server host when using HTTP transport (default: "127.0.0.1")
- `RUST_LOG` - Tracing filter levels (e.g., `obsidian_mcp=debug,rmcp=warn`)

**Secrets location:**
- Environment variables (typically sourced from `.env` file via dotenvy in tests, or Claude Desktop configuration)
- `.env` file present but contents not version-controlled (git-ignored)

## Webhooks & Callbacks

**Incoming:**
- None - This is an MCP server that responds to tool calls from MCP clients

**Outgoing:**
- None - No webhooks or event callbacks to external services

## Transport Protocols

**MCP Transport:**
- **Stdio (default)**: Standard I/O streams for bidirectional JSON-RPC communication with Claude Desktop or compatible MCP client
  - Implemented via `rmcp::transport::io::stdio()` in `src/main.rs`
  - Controlled by `--transport stdio` or default behavior

- **Streamable HTTP (optional)**: JSON-RPC over HTTP at `/mcp` endpoint
  - Implemented via `rmcp::transport::streamable_http_server` from rmcp 0.12
  - Controlled by `--transport http`
  - Configuration: `StreamableHttpServerConfig` with `stateful_mode: true`
  - Session management: `LocalSessionManager` for handling client sessions
  - Graceful shutdown: `CancellationToken` tied to SIGINT handling

**Obsidian REST API Transport:**
- HTTP/HTTPS (all requests to Obsidian use HTTP methods: GET, PUT, POST, PATCH, DELETE)
- URL patterns in `src/client.rs`:
  - `/` - Server info
  - `/vault/{path}` - Note CRUD operations
  - `/vault/{path}/` - Directory listing
  - `/search/simple/` - Simple search
  - `/search/` - Dataview query search (custom Content-Type: `application/vnd.olrapi.dataview.dql+txt`)
  - `/commands/` - Command listing
  - `/commands/{id}/` - Command execution
  - `/open/{filename}` - File open in UI
  - `/periodic/{period}/[{year}/{month}/{day}/]` - Periodic note operations

## API Integration Details

**HTTP Status Handling:**
- Success: 200-299 status codes accepted
- Error: Non-success responses mapped to `AppError::Api { status, body }` in `src/client.rs` via `check_response()` helper
- Error messages include both HTTP status code and response body

**Content Types:**
- Default: `text/markdown` for note content
- Special: `application/json` for frontmatter array values (configured via `content_type` in `PatchParams`)
- Search: `application/vnd.olrapi.dataview.dql+txt` for Dataview query syntax

**Request Headers:**
- `Authorization: Bearer {api_key}` - All requests
- `Content-Type: text/markdown` or `application/json` - For write operations
- `Accept: text/markdown` or `application/json` - For read operations
- PATCH-specific headers: `Operation`, `Target-Type`, `Target`, `Target-Delimiter`, `Trim-Target-Whitespace`, `Create-Target-If-Missing`

---

*Integration audit: 2026-03-10*
