# Testing Patterns

**Analysis Date:** 2026-03-10

## Test Framework

**Runner:**
- `cargo test` (Rust standard test framework)
- Config: No separate config file; uses Cargo.toml `[dev-dependencies]`

**Assertion Library:**
- Standard `assert!`, `assert_eq!`, `assert_ne!` macros (built-in)
- No external assertion library

**Run Commands:**
```bash
just unit-test             # Run unit tests (--lib only, no integration tests)
just test-verbose          # Run all tests with output (--nocapture)
just e2e                   # Run e2e tests with serial execution (--test-threads=1)
just coverage              # Run unit tests with ≥85% line coverage threshold
just coverage-report       # Generate HTML coverage report (llvm-cov)
just lint                  # Format check + clippy (lint)
```

## Test File Organization

**Location:**
- Unit tests: co-located with source code in `#[cfg(test)]` modules within `.rs` files
- Integration tests: separate `tests/` directory files
- Examples: `src/server.rs` has `#[cfg(test)] mod tests { ... }` at end of file

**Naming:**
- Unit test functions: `test_<what_being_tested>` format (e.g., `test_url_concatenates_base_and_path`)
- Integration test functions: `e2e_<flow_description>` format (e.g., `e2e_create_and_read_note`)
- Test modules: `tests` (plural)
- Test helpers: descriptive names like `make_server`, `setup`, `cleanup`

**File Structure:**
```
src/
├── server.rs              # Contains server impl + #[cfg(test)] mod tests
├── client.rs              # Contains client impl + #[cfg(test)] mod tests
├── types.rs               # Contains type defs + #[cfg(test)] mod tests
├── error.rs               # Contains error enum + #[cfg(test)] mod tests
└── main.rs                # Contains CLI + #[cfg(test)] mod tests

tests/
├── integration_test.rs    # E2E tests against real Obsidian API
└── test_stdio.rs          # Stdio transport integration test
```

## Test Structure

**Suite Organization:**

Unit tests (co-located example from `src/server.rs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;
    use serde_json::Value;
    use wiremock::matchers::{body_string, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn get_field_description(schema: &schemars::Schema, field: &str) -> String {
        // Helper function
    }

    async fn make_server(mock: &MockServer) -> ObsidianServer {
        // Setup function
    }

    fn text_content(result: &CallToolResult) -> &str {
        // Assertion helper
    }

    #[test]
    fn patch_note_target_field_describes_nested_heading_path() {
        // Test body
    }

    #[tokio::test]
    async fn read_note_returns_content() {
        // Async test body
    }
}
```

Integration tests (from `tests/integration_test.rs`):
```rust
// Global test helpers
fn init_dotenv() {
    INIT_DOTENV.call_once(|| {
        let _ = dotenvy::dotenv();
    });
}

macro_rules! require_api_key {
    () => {
        // Macro to skip tests if API key not set
    }
}

async fn setup(api_key: &str) -> (...) {
    // Full test harness setup
}

async fn cleanup(raw_client: &ObsidianClient) {
    // Cleanup after tests
}

#[tokio::test]
async fn e2e_create_and_read_note() {
    // Test body
}
```

**Patterns:**
- Setup via helper functions (`make_server`, `setup`, `mock_client`)
- Teardown via cleanup functions (`cleanup`) or cancellation tokens
- Assertion helpers extract common patterns (`text_content`, `first_text`)
- Macros used for conditional skipping (`require_api_key!`)

## Mocking

**Framework:** `wiremock` crate (version 0.6)

**Patterns (from `src/client.rs` unit tests):**

Basic mock setup:
```rust
let server = MockServer::start().await;
Mock::given(method("GET"))
    .and(path("/vault/folder/note.md"))
    .and(header("Authorization", "Bearer test-key"))
    .respond_with(ResponseTemplate::new(200).set_body_string("# Hello"))
    .mount(&server)
    .await;

let client = mock_client(server.uri());
let result = client.read_note("folder/note.md").await.unwrap();
```

HTTP method matching:
```rust
Mock::given(method("PUT"))        // Matches HTTP method
    .and(path("/vault/new.md"))   // Matches request path
    .and(header("Content-Type", "text/markdown"))  // Matches header
    .and(body_string("# New note"))  // Matches body content
    .respond_with(ResponseTemplate::new(204))
```

Query parameter matching:
```rust
Mock::given(method("POST"))
    .and(path("/search/simple/"))
    .and(query_param("query", "test"))  // Matches query string
    .respond_with(...)
```

**What to Mock:**
- HTTP requests from `reqwest::Client`
- Obsidian REST API responses
- Error conditions (404, 500, etc.)
- Headers to verify request format

**What NOT to Mock:**
- Internal function calls (test complete methods)
- Serialization/deserialization (test real types)
- Error enum construction (test real error types)

## Fixtures and Factories

**Test Data:**

Simple factory functions (from `src/client.rs`):
```rust
fn make_client() -> ObsidianClient {
    ObsidianClient::new(
        "https://localhost:27124".to_string(),
        "test-api-key".to_string(),
    )
}

fn mock_client(uri: String) -> ObsidianClient {
    ObsidianClient::new(uri, "test-key".to_string())
}
```

Argument builders (from `src/server.rs`):
```rust
let result = server
    .patch_note(Parameters(PatchNoteArgs {
        path: "note.md".to_string(),
        operation: Operation::Append,
        target_type: TargetType::Heading,
        target: "Section".to_string(),
        target_delimiter: None,
        trim_target_whitespace: None,
        create_target_if_missing: None,
        content_type: None,
        content: "new text\n".to_string(),
    }))
    .await
```

JSON fixtures (from `tests/integration_test.rs`):
```rust
let args = json!({"path": note_path, "content": content});
```

**Location:**
- Factories defined at top of `#[cfg(test)]` modules
- No separate fixtures directory; inline fixture data via JSON macros
- Parameter structs used as explicit fixtures

## Coverage

**Requirements:**
- ≥85% line coverage threshold enforced for library code (`--lib`)
- Run via `just coverage` command

**View Coverage:**
```bash
just coverage              # Run tests and fail if coverage < 85%
just coverage-report       # Generate HTML report (opens in browser after generation)
```

**Measured:**
- Line coverage only (not branch coverage)
- Integration tests excluded from coverage (`--lib` flag limits to library)
- Excludes e2e tests (they require external Obsidian instance)

## Test Types

**Unit Tests:**
- Scope: Individual modules and functions
- Location: Co-located in `#[cfg(test)]` sections of `src/server.rs`, `src/client.rs`, `src/types.rs`, `src/error.rs`, `src/main.rs`
- Approach:
  - Test happy paths and error conditions
  - Use wiremock to mock HTTP calls
  - Verify struct field descriptions via schema introspection
  - Test type conversions and display formatting
- Coverage: ~30 unit tests across codebase

**Integration Tests:**
- Scope: Full tool lifecycle against real Obsidian API
- Location: `tests/integration_test.rs`
- Approach:
  - Spin up full MCP server (stdio or HTTP transport)
  - Call tools via MCP client
  - Test against real Obsidian vault
  - Cleanup via `cleanup()` helper
  - Skip if `OBSIDIAN_API_KEY` env var not set
- Tests:
  - CRUD operations: create, read, append, delete
  - Patch operations: heading/block/frontmatter patching
  - Query operations: list_files, search, search_query
  - Periodic notes: daily, weekly, monthly, etc.
  - Tool listing and command execution
- Serialization: `@[serial]` attribute via `serial_test` crate to run tests sequentially

**E2E/Stdio Tests:**
- Scope: Stdio transport communication
- Location: `tests/test_stdio.rs`
- Approach:
  - Spawn obsidian-mcp binary as subprocess
  - Communicate via stdin/stdout with JSON-RPC messages
  - Verify protocol compliance
  - Skip if `OBSIDIAN_API_KEY` env var not set

**No separate E2E test framework:**
- Integration tests serve as E2E tests
- Uses actual MCP SDK client + HTTP transport

## Common Patterns

**Async Testing:**

Using `#[tokio::test]` attribute:
```rust
#[tokio::test]
async fn read_note_returns_content() {
    let mock = MockServer::start().await;
    // ... setup and assertions
}
```

**Error Testing:**

Testing error conditions with `unwrap_err()`:
```rust
#[test]
fn invalid_operation_fails_deserialization() {
    let result: Result<Operation, _> = serde_json::from_str("\"invalid\"");
    assert!(result.is_err());
}
```

Verifying error messages:
```rust
#[tokio::test]
async fn search_query_rejects_non_table_queries() {
    let mock = MockServer::start().await;
    let server = make_server(&mock).await;
    let err = server
        .search_query(Parameters(SearchArgs {
            query: "LIST FROM \"folder\"".to_string(),
        }))
        .await
        .unwrap_err();
    assert!(err.message.contains("Only TABLE queries"));
}
```

**Conditional Test Skipping:**

Skip if environment variable not set:
```rust
macro_rules! require_api_key {
    () => {
        match api_key() {
            Some(key) => key,
            None => {
                eprintln!("OBSIDIAN_API_KEY not set, skipping test");
                return;
            }
        }
    };
}

#[tokio::test]
async fn e2e_create_and_read_note() {
    let key = require_api_key!();
    let (client, cancel, raw) = setup(&key).await;
    // Test body
}
```

**Environment Loading:**

Loading `.env` file once at startup:
```rust
static INIT_DOTENV: Once = Once::new();

fn init_dotenv() {
    INIT_DOTENV.call_once(|| {
        let _ = dotenvy::dotenv();
    });
}
```

---

*Testing analysis: 2026-03-10*
