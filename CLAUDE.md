# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

Uses [just](https://github.com/casey/just) for task orchestration and [uv](https://docs.astral.sh/uv/) for Python project management.

```bash
just run                 # Run the server (pass extra args after --)
just fmt                 # Format code (ruff)
just lint                # fmt-check + ruff check
just lint-fix            # Auto-fix lint issues
just unit-test           # Run unit tests
just test-verbose        # Run tests with output
just e2e                 # Run e2e tests (requires OBSIDIAN_API_KEY, see docs/e2e-testing.md)
just coverage            # Run tests with ≥85% line coverage threshold
just coverage-report     # Generate HTML coverage report
just clean               # Clean build artifacts
just deploy              # __check + install as uv tool
```

Unit tests in `tests/test_client.py` use respx to mock the Obsidian REST API; `tests/test_types.py` and `tests/test_errors.py` have unit tests for shared types and error handling. E2e tests in `tests/test_e2e.py` run against the real Obsidian Local REST API (see `docs/e2e-testing.md` for prerequisites).

## Architecture

This is an MCP (Model Context Protocol) server that bridges AI assistants to Obsidian vaults via the [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) plugin. It supports both stdio (default) and Streamable HTTP transport, selectable via `--transport`.

**Source files:**

- `src/obsidian_mcp/__init__.py` — Package marker.
- `src/obsidian_mcp/__main__.py` — CLI entrypoint (click), server startup, connectivity check. Supports `--transport stdio` (default) and `--transport http` (mounts MCP endpoint at `/mcp`).
- `src/obsidian_mcp/server.py` — FastMCP server instance with all 16 MCP tools defined using `@mcp.tool()` decorators. Tool arguments are plain Python type hints (auto-generates JSON Schema via Pydantic).
- `src/obsidian_mcp/client.py` — `ObsidianClient` wraps `httpx.AsyncClient` to call the Obsidian REST API. Maps HTTP methods to vault operations (GET=read, PUT=create, POST=append, PATCH=partial update, DELETE=delete). Accepts invalid TLS certs since Obsidian's local API uses self-signed certs. Bearer token is pre-formatted in the constructor; uses `_check_response()` helper to deduplicate error handling.
- `src/obsidian_mcp/types.py` — Shared types for the v3 PATCH API: `Operation` (StrEnum), `TargetType` (StrEnum), and `PatchParams` (dataclass).
- `src/obsidian_mcp/errors.py` — `AppError` exception hierarchy: `HttpError`, `ApiError`, `JsonError`.

**Key dependencies:** `mcp` (official MCP Python SDK with FastMCP), `httpx` (async HTTP client), `click` (CLI), `pydantic` (JSON Schema generation for tool args).

## Adding a New Tool

1. Add an `async def` with `@mcp.tool()` decorator in `server.py`
2. Use plain Python type hints for arguments (Pydantic auto-generates JSON Schema)
3. If the tool needs a new API call, add the corresponding method to `ObsidianClient` in `client.py`

## Process Rules

- **TDD is mandatory.** Every implementation task must follow: write failing test → verify it fails → implement → verify it passes → commit. Never write plans or code without tests.
- **Never skip reviews** in subagent-driven development — spec compliance + code quality reviews are both required for every task.
- **Include TDD instructions** in every implementer subagent prompt. Invoke the test-driven-development skill.
