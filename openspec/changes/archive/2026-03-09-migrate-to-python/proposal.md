## Why

Rewrite the entire project from Rust to Python for easier maintenance, faster iteration, and broader contributor accessibility. The project is a straightforward HTTP-to-MCP bridge with no performance-critical paths — Python with async is more than sufficient.

## What Changes

- **BREAKING**: Replace entire Rust codebase with Python implementation
- Replace `rmcp` with official `mcp` Python SDK for MCP protocol handling
- Replace `reqwest` with `httpx` for async HTTP client
- Replace `clap` with `click` for CLI argument parsing
- Replace `axum` with MCP SDK's built-in Streamable HTTP server
- Replace `wiremock` unit tests with `respx` + `pytest`
- Replace `cargo` with `uv` for project and dependency management
- Replace Rust `justfile` recipes with Python equivalents
- Delete all Rust source files, `Cargo.toml`, `Cargo.lock`
- All 16 MCP tools preserved with identical behavior
- All CLI arguments preserved (`--api-url`, `--api-key`, `--port`, `--host`) with same env var mappings and defaults

## Capabilities

### New Capabilities

_(none — this is a rewrite, not a feature change)_

### Modified Capabilities

- `client`: Implementation changes from Rust/reqwest to Python/httpx. All HTTP method mappings, headers, URL patterns, and behavior contracts remain identical.
- `server`: Implementation changes from Rust/rmcp to Python/mcp SDK. All 16 tools, their args, descriptions, and return formats remain identical.
- `types`: Implementation changes from Rust enums/structs to Python enums/Pydantic models. Same domain types, same validation rules.
- `transport`: Implementation changes from Rust/axum to Python/mcp SDK built-in server. Same CLI args, same startup/shutdown behavior, same `/mcp` endpoint.

## Impact

- **Code**: Entire `src/` directory replaced with `src/obsidian_mcp/` Python package
- **Tests**: `tests/` rewritten with pytest; same coverage expectations (≥85%)
- **Build**: `Cargo.toml` → `pyproject.toml` with uv; `justfile` recipes updated
- **Dependencies**: Entire dependency tree changes (Rust crates → Python packages)
- **Deployment**: Binary distribution → Python package (`uv run` or `pip install`)
- **Specs**: All 4 specs updated with Python equivalents (no behavioral changes)
