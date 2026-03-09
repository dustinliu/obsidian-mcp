# Transport Spec

## Purpose

Handles CLI argument parsing, HTTP client setup, connectivity check, and wiring of the MCP transport layer. Defined in `src/obsidian_mcp/__main__.py`.

## Public Interface

```python
@click.command()
@click.option("--api-url", envvar="OBSIDIAN_API_URL", default="https://127.0.0.1:27124")
@click.option("--api-key", envvar="OBSIDIAN_API_KEY", required=True)
@click.option("--port",    envvar="MCP_PORT",         default=3000, type=int)
@click.option("--host",    envvar="MCP_HOST",         default="127.0.0.1")
def main(api_url: str, api_key: str, port: int, host: str) -> None: ...
```

## Behavior Contracts

### Startup sequence

1. Parse CLI args via `click.command()` / `click.option()`.
2. Configure logging with `logging.basicConfig()`.
3. Run async entrypoint `_run()` via `asyncio.run()`.
4. In `_run()`:
   a. Build `ObsidianClient` with `api_url` + `api_key`, used as async context manager (`async with`).
   b. Call `client.server_info()` to verify connectivity.
      - **Success**: log and continue.
      - **Failure**: log error and `sys.exit(1)`.
   c. Call `set_client(client)` to make the client available to tool functions.
   d. Configure `mcp.settings.host` and `mcp.settings.port`.
   e. Call `mcp.run_streamable_http_async()` to start the MCP server.

### Shutdown sequence

1. Process termination signal received.
2. `httpx.AsyncClient` is closed via the `async with` context manager.
3. Process exits cleanly.

### MCP transport configuration

- Transport: **Streamable HTTP** (via FastMCP's `run_streamable_http_async()`).
- Host/port configured on `mcp.settings` before starting.

## Invariants

- `OBSIDIAN_API_KEY` must be provided; process fails to start without it (enforced by `click.option(required=True)`).
- The server refuses to start if Obsidian is unreachable at startup.
- Listening address is `{host}:{port}`; both have environment variable overrides.

## Integration Points

- Constructs `ObsidianClient` and manages its lifecycle via `async with`.
- Calls `set_client()` from `server.py` to wire the client into tool functions.
- Imports the `mcp` FastMCP instance from `server.py` and configures its settings.

## Constraints

- No TLS on the MCP endpoint itself; relies on the host network for security.
- `ObsidianClient` lifetime is tied to the `async with` block; the client is closed when the server stops.
