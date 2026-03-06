# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

Uses [cargo-make](https://github.com/sagiegurari/cargo-make) for task orchestration.

```bash
cargo make build               # Debug build
cargo make build-release       # Release build
cargo make run                 # Run (pass args via CARGO_MAKE_CARGO_ARGS)
cargo make fmt                 # Format
cargo make clippy              # Lint (warnings as errors)
cargo make lint                # fmt-check + clippy
cargo make test                # Run all tests
cargo make test-verbose        # Run tests with output
cargo make coverage            # Run tests with ≥85% line coverage threshold
cargo make coverage-report     # Generate HTML coverage report
cargo make check               # lint + test + build
```

Tests use wiremock to mock the Obsidian REST API. Unit tests live in `src/client.rs` and `src/server.rs`; e2e MCP tests live in `tests/integration_test.rs` (exercises the full MCP stack: MCP client → HTTP → Axum → ObsidianServer → ObsidianClient → wiremock).

## Architecture

This is an MCP (Model Context Protocol) server that bridges AI assistants to Obsidian vaults via the [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) plugin. It uses Streamable HTTP transport (not stdio).

**Source files:**

- `src/lib.rs` — Re-exports `client`, `error`, and `server` as public modules.
- `src/main.rs` — CLI parsing (clap), Axum HTTP server setup, MCP transport wiring. The MCP endpoint is mounted at `/mcp`.
- `src/server.rs` — `ObsidianServer` implements `ServerHandler` from the `rmcp` crate. All 16 MCP tools are defined here using `#[tool]` / `#[tool_router]` / `#[tool_handler]` proc macros. Each tool method deserializes args from a `Parameters<T>` wrapper where `T` is a `Deserialize + JsonSchema` struct defined in the same file. Uses `to_mcp_error()` helper to convert errors.
- `src/client.rs` — `ObsidianClient` wraps `reqwest::Client` to call the Obsidian REST API. Maps HTTP methods to vault operations (GET=read, PUT=create, POST=append, PATCH=partial update, DELETE=delete). Accepts invalid TLS certs since Obsidian's local API uses self-signed certs. Bearer token is pre-formatted in the constructor; uses `check_response()` helper to deduplicate error handling.
- `src/error.rs` — `AppError` enum using `thiserror`.

**Key dependencies:** `rmcp` (MCP protocol SDK with macros), `axum` (HTTP server), `reqwest` (HTTP client), `clap` (CLI), `schemars` (JSON Schema generation for tool args).

## Adding a New Tool

1. Add an args struct with `Deserialize + JsonSchema` in `server.rs`
2. Add an `async fn` method inside the `#[tool_router] impl ObsidianServer` block with a `#[tool(description = "...")]` attribute
3. If the tool needs a new API call, add the corresponding method to `ObsidianClient` in `client.rs`

## Process Rules

- **TDD is mandatory.** Every implementation task must follow: write failing test → verify it fails → implement → verify it passes → commit. Never write plans or code without tests.
- **Never skip reviews** in subagent-driven development — spec compliance + code quality reviews are both required for every task.
- **Include TDD instructions** in every implementer subagent prompt. Invoke the test-driven-development skill.
