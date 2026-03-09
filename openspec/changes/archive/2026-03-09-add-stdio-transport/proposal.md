## Why

Claude Desktop and similar MCP clients launch servers as subprocesses communicating via stdio. The server currently only supports Streamable HTTP transport, preventing use with these clients.

## What Changes

- Add `--transport` CLI option with choices `stdio` (default) and `http`. **BREAKING**: default transport changes from HTTP to stdio.
- In stdio mode, call `mcp.run_stdio_async()` instead of `mcp.run_streamable_http_async()`.
- `--host` and `--port` options only apply to HTTP transport; ignored in stdio mode.
- Update `just deploy` recipe to explicitly pass `--transport http` to preserve existing deployment behavior.

## Capabilities

### New Capabilities

(none — this extends the existing transport capability)

### Modified Capabilities

- `transport`: Add stdio transport support and make it the default. HTTP transport becomes opt-in via `--transport http`.

## Impact

- `src/obsidian_mcp/__main__.py` — add `--transport` option, branch on transport mode
- `justfile` — update `deploy` recipe
- No changes to `server.py`, `client.py`, `types.py`, or `errors.py`
