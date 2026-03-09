## MODIFIED Requirements

### Requirement: CLI arguments
The CLI SHALL accept 4 arguments via `click`: `--api-url` (env: `OBSIDIAN_API_URL`, default: `"https://127.0.0.1:27124"`), `--api-key` (env: `OBSIDIAN_API_KEY`, required), `--port` (env: `MCP_PORT`, default: `3000`), `--host` (env: `MCP_HOST`, default: `"127.0.0.1"`).

#### Scenario: Default values
- **WHEN** the CLI is invoked with only `--api-key`
- **THEN** `api_url` SHALL be `"https://127.0.0.1:27124"`, `port` SHALL be `3000`, `host` SHALL be `"127.0.0.1"`

#### Scenario: Environment variable override
- **WHEN** `OBSIDIAN_API_KEY` is set as an environment variable
- **THEN** `--api-key` flag SHALL not be required

### Requirement: Startup sequence
The server SHALL verify Obsidian connectivity at startup by calling `server_info()`. If unreachable, it SHALL log the error and exit with code 1.

#### Scenario: Obsidian unreachable at startup
- **WHEN** the server starts and Obsidian is not reachable
- **THEN** the process SHALL exit with code 1

### Requirement: MCP transport
The server SHALL use Streamable HTTP transport via the `mcp` Python SDK. The MCP endpoint SHALL be mounted at `/mcp`.

#### Scenario: MCP endpoint path
- **WHEN** an MCP client connects
- **THEN** it SHALL connect to the `/mcp` endpoint

### Requirement: Graceful shutdown
The server SHALL handle `SIGINT` (Ctrl-C) and shut down gracefully, draining in-flight requests.

#### Scenario: Ctrl-C shutdown
- **WHEN** `SIGINT` is received
- **THEN** the server SHALL stop accepting new connections and drain existing requests before exiting

### Requirement: Project management with uv
The project SHALL use `uv` for dependency management with a `pyproject.toml`. It SHALL be runnable via `uv run obsidian-mcp` or `python -m obsidian_mcp`.

#### Scenario: Run via uv
- **WHEN** `uv run obsidian-mcp --api-key <key>` is executed
- **THEN** the MCP server SHALL start on the default host and port
