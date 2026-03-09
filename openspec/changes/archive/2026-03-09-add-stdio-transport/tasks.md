## 1. CLI and Transport

- [x] 1.1 Add `--transport` click option with choices `stdio`/`http`, default `stdio`, envvar `MCP_TRANSPORT`
- [x] 1.2 Branch `_run()` on transport: call `mcp.run_stdio_async()` for stdio, existing HTTP logic for http
- [x] 1.3 Only configure `mcp.settings.host`/`mcp.settings.port` when transport is `http`

## 2. Build and Deploy

- [x] 2.1 Update `just deploy` recipe to pass `--transport http` (N/A: deploy only installs, transport is a runtime flag)

## 3. Documentation

- [x] 3.1 Update CLAUDE.md with new `--transport` option and default behavior
- [x] 3.2 Update README.md usage section with both transport modes and Claude Desktop config example

## 4. Tests

- [x] 4.1 Add unit tests for transport selection (stdio default, explicit http, explicit stdio, env var override)
