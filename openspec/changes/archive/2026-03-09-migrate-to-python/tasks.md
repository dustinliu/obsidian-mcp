## 1. Project Setup

- [x] 1.1 Delete all Rust files (src/, Cargo.toml, Cargo.lock, tests/, .cargo/)
- [x] 1.2 Initialize uv project with pyproject.toml (Python >=3.12, src layout)
- [x] 1.3 Add dependencies: mcp, httpx, click, pydantic
- [x] 1.4 Add dev dependencies: pytest, pytest-asyncio, respx, pytest-cov
- [x] 1.5 Create src/obsidian_mcp/ package with __init__.py

## 2. Core Types & Errors

- [x] 2.1 Implement errors.py: AppError, HttpError, ApiError, JsonError with correct display formats
- [x] 2.2 Implement types.py: Operation (StrEnum), TargetType (StrEnum), PatchParams (dataclass)
- [x] 2.3 Write tests for errors (test_errors.py): display formats, inheritance
- [x] 2.4 Write tests for types (test_types.py): enum values, invalid values, PatchParams defaults

## 3. HTTP Client

- [x] 3.1 Implement client.py: ObsidianClient with httpx.AsyncClient, verify=False, bearer token
- [x] 3.2 Implement vault methods: read_note, create_note, append_note, patch_note, delete_note, list_files
- [x] 3.3 Implement search methods: search_simple (query param), search_query (DQL body)
- [x] 3.4 Implement command methods: list_commands, execute_command
- [x] 3.5 Implement UI method: open_file
- [x] 3.6 Implement periodic note methods: get, update, append, patch with periodic_url helper
- [x] 3.7 Implement server_info method
- [x] 3.8 Implement async context manager (__aenter__/__aexit__)
- [x] 3.9 Write unit tests (test_client.py) with respx: all 15 API methods + error handling + URL helpers

## 4. MCP Server

- [x] 4.1 Implement server.py: create MCP server instance with tool definitions for all 16 tools
- [x] 4.2 Implement tool arg models as Pydantic BaseModel classes with Field descriptions
- [x] 4.3 Implement search_query TABLE validation
- [x] 4.4 Write unit tests for server tools (mock ObsidianClient, verify args/returns)

## 5. CLI & Transport

- [x] 5.1 Implement __main__.py: click CLI with 4 args (api-url, api-key, port, host) and env var support
- [x] 5.2 Wire startup sequence: create client, verify connectivity, start MCP server on /mcp
- [x] 5.3 Implement graceful shutdown on SIGINT

## 6. Build & Tooling

- [x] 6.1 Update justfile with Python recipes: build, run, fmt, lint, test, coverage, deploy
- [x] 6.2 Configure ruff for formatting and linting in pyproject.toml
- [x] 6.3 Verify coverage threshold ≥85%

## 7. E2E Tests

- [x] 7.1 Port e2e tests to pytest: setup/cleanup fixtures, all 14 test scenarios
- [ ] 7.2 Verify e2e tests pass against real Obsidian (requires OBSIDIAN_API_KEY)

## 8. Documentation

- [x] 8.1 Update CLAUDE.md with Python build commands and architecture
- [x] 8.2 Update README.md with Python installation and usage
- [x] 8.3 Update OpenSpec main specs to reflect Python implementation
