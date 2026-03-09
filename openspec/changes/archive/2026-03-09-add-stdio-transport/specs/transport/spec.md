## ADDED Requirements

### Requirement: Transport selection via CLI
The CLI SHALL accept a `--transport` option with choices `stdio` and `http`, defaulting to `stdio`. The option SHALL also be configurable via the `MCP_TRANSPORT` environment variable.

#### Scenario: Default transport is stdio
- **WHEN** the server is started without `--transport`
- **THEN** the server SHALL use stdio transport via `mcp.run_stdio_async()`

#### Scenario: Explicit HTTP transport
- **WHEN** the server is started with `--transport http`
- **THEN** the server SHALL use Streamable HTTP transport via `mcp.run_streamable_http_async()` on the configured host and port

#### Scenario: Explicit stdio transport
- **WHEN** the server is started with `--transport stdio`
- **THEN** the server SHALL use stdio transport via `mcp.run_stdio_async()`

#### Scenario: Environment variable override
- **WHEN** `MCP_TRANSPORT` is set to `http` and `--transport` is not provided
- **THEN** the server SHALL use HTTP transport

## MODIFIED Requirements

### Requirement: Startup sequence
1. Parse CLI args via `click.command()` / `click.option()`.
2. Configure logging with `logging.basicConfig()`.
3. Run async entrypoint `_run()` via `asyncio.run()`.
4. In `_run()`:
   a. Build `ObsidianClient` with `api_url` + `api_key`, used as async context manager (`async with`).
   b. Call `client.server_info()` to verify connectivity.
      - **Success**: log and continue.
      - **Failure**: log error and `sys.exit(1)`.
   c. Call `set_client(client)` to make the client available to tool functions.
   d. Branch on transport:
      - `stdio`: Call `mcp.run_stdio_async()`.
      - `http`: Configure `mcp.settings.host` and `mcp.settings.port`, then call `mcp.run_streamable_http_async()`.

#### Scenario: Stdio startup
- **WHEN** the server starts with stdio transport
- **THEN** it SHALL verify Obsidian connectivity, then call `mcp.run_stdio_async()` without configuring host/port

#### Scenario: HTTP startup
- **WHEN** the server starts with HTTP transport
- **THEN** it SHALL verify Obsidian connectivity, configure host/port on `mcp.settings`, then call `mcp.run_streamable_http_async()`

### Requirement: MCP transport configuration
- Transport: **stdio** (default, via `mcp.run_stdio_async()`) or **Streamable HTTP** (via `mcp.run_streamable_http_async()`).
- Host/port configured on `mcp.settings` only for HTTP transport.

#### Scenario: Host and port in HTTP mode
- **WHEN** transport is `http`
- **THEN** `mcp.settings.host` and `mcp.settings.port` SHALL be set before starting

#### Scenario: Host and port in stdio mode
- **WHEN** transport is `stdio`
- **THEN** `mcp.settings.host` and `mcp.settings.port` SHALL NOT be configured
