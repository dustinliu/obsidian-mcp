## Context

obsidian-mcp is an MCP server bridging AI assistants to Obsidian vaults via the Local REST API plugin. Currently implemented in Rust (rmcp + reqwest + axum + clap). The project is a thin HTTP-to-MCP bridge with no compute-intensive logic — Python is a natural fit.

Current state: 16 MCP tools, all working, with unit tests (wiremock) and e2e tests against real Obsidian.

## Goals / Non-Goals

**Goals:**
- 1:1 functional parity with the Rust implementation (all 16 tools, same CLI args, same behavior)
- Pythonic architecture using modern Python async patterns
- `uv` for project management, `pytest` for testing
- Maintain ≥85% test coverage
- Clean migration: delete all Rust code, no hybrid state

**Non-Goals:**
- Adding new tools or features during migration
- Changing the Obsidian REST API integration behavior
- Supporting stdio transport (Streamable HTTP only, same as current)
- Backwards compatibility with Rust binary distribution

## Decisions

### 1. MCP SDK: `mcp` (official Python SDK)

The official `mcp` Python SDK from Anthropic supports Streamable HTTP transport and provides decorator-based tool definition with Pydantic models for automatic JSON Schema generation.

**Alternative considered**: `fastmcp` — higher-level wrapper, but adds abstraction we don't need. The official SDK is closer to metal and well-maintained.

### 2. HTTP Client: `httpx`

Async-native, supports `verify=False` for self-signed certs, clean API.

**Alternative considered**: `aiohttp` — more verbose, `httpx` has a more intuitive requests-like API.

### 3. CLI: `click`

Lightweight, mature, supports env var fallbacks natively. Only 4 CLI args — no need for anything heavier.

**Alternative considered**: `typer` — adds a dependency layer on top of click for type-hint-driven CLI. Overkill for 4 parameters.

### 4. Project structure: `src` layout with `uv`

```
src/obsidian_mcp/
├── __init__.py        # package version
├── __main__.py        # CLI entrypoint (click)
├── server.py          # MCP server + 16 tool definitions
├── client.py          # ObsidianClient (httpx)
├── types.py           # Operation, TargetType, PatchParams
└── errors.py          # AppError hierarchy
```

Maps 1:1 with Rust modules. `__main__.py` replaces `main.rs` for `python -m obsidian_mcp` support.

### 5. Testing: `pytest` + `respx`

- `respx` mocks `httpx` requests at the transport level (same concept as wiremock)
- `pytest-asyncio` for async test support
- `pytest-cov` for coverage with ≥85% threshold
- E2e tests rewritten with same patterns: spin up real server, connect MCP client

### 6. Error handling: Custom exception classes

```python
class AppError(Exception): ...
class HttpError(AppError): ...
class ApiError(AppError): status, body
class JsonError(AppError): ...
```

Direct mapping from Rust `AppError` enum variants.

### 7. Type definitions: Python enums + Pydantic

- `Operation` and `TargetType` as `str` enums (StrEnum)
- Tool arg structs as Pydantic `BaseModel` with `Field()` descriptions
- MCP SDK auto-generates JSON Schema from Pydantic models

## Risks / Trade-offs

- **[Risk] MCP Python SDK Streamable HTTP maturity** → The SDK is actively maintained by Anthropic; Streamable HTTP is a primary transport. Mitigated by testing early.
- **[Risk] Python async performance vs Rust** → Irrelevant for this use case. The bottleneck is Obsidian's REST API latency, not the bridge.
- **[Trade-off] Binary distribution lost** → Rust produced a single binary; Python requires a Python runtime. Mitigated by `uv` which can manage the Python version automatically. Can add `pyinstaller` later if needed.
- **[Risk] Behavioral drift during rewrite** → Mitigated by keeping all existing specs as the source of truth and maintaining the same test scenarios.

## Migration Plan

1. Delete all Rust source files (`src/`, `Cargo.toml`, `Cargo.lock`, `tests/`)
2. Initialize `uv` project with `pyproject.toml`
3. Implement Python modules in dependency order: errors → types → client → server → main
4. Port unit tests (client, types, errors) with same coverage
5. Port e2e tests
6. Update `justfile` with Python recipes
7. Update `CLAUDE.md` and `README.md`
8. Update OpenSpec specs to reflect Python implementation
