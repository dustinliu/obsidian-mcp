# Coding Conventions

**Analysis Date:** 2026-03-10

## Naming Patterns

**Files:**
- Rust modules follow snake_case: `src/server.rs`, `src/client.rs`, `src/types.rs`, `src/error.rs`
- Test files in `tests/` directory use snake_case: `integration_test.rs`, `test_stdio.rs`

**Functions:**
- All functions use snake_case: `read_note`, `create_note`, `patch_note`, `delete_note`, `list_files`
- Async functions marked with `async` keyword
- Test helper functions are descriptive: `make_server`, `make_client`, `mock_client`, `text_content`, `first_text`
- Private helper functions use underscore prefix pattern: `prepare_patch_body`, `check_response`, `periodic_url`, `url`

**Variables:**
- Local variables and parameters use snake_case: `api_key`, `bearer_token`, `mock_server`, `note_path`
- Constants in uppercase where present
- Struct field names match database/API naming conventions when representing external data

**Types:**
- Struct names use PascalCase: `ObsidianClient`, `ObsidianServer`, `AppError`, `ReadNoteArgs`, `PatchNoteArgs`
- Enum variants use PascalCase: `Operation::Append`, `TargetType::Heading`
- Type argument structs suffixed with `Args`: `ReadNoteArgs`, `CreateNoteArgs`, `AppendNoteArgs`, `PatchNoteArgs`, `DeleteNoteArgs`, `ListFilesArgs`, `SearchArgs`

## Code Style

**Formatting:**
- Tool: `cargo fmt` (Rust standard formatter)
- Standard 4-space indentation
- Line length follows rustfmt defaults
- All code must pass `cargo fmt --check` before commit

**Linting:**
- Tool: `cargo clippy`
- Configuration: Warnings treated as errors (`-D warnings`)
- All code must pass `cargo clippy -- -D warnings` before commit
- Lint checks run via `just lint` command

## Import Organization

**Order:**
1. Standard library imports (`use std::...`)
2. External crate imports (by category: async runtime, HTTP, serialization, etc.)
3. Internal crate imports (`use crate::...`)
4. Test-specific imports in `#[cfg(test)]` sections

**Examples from codebase:**
```rust
// src/server.rs order
use std::sync::Arc;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::ObsidianClient;
use crate::types::{Operation, PatchParams, TargetType};
```

**Path Aliases:**
- Not currently used (standard crate-relative imports preferred)

## Error Handling

**Patterns:**
- Custom error enum `AppError` using `thiserror` crate with `#[error]` derive macro in `src/error.rs`
- Error variants include `Http`, `Api`, `Json`
- HTTP errors use `#[from]` for automatic conversion: `#[error(...)] Http(#[from] reqwest::Error)`
- Error conversion helper `to_mcp_error()` for converting `AppError` to MCP protocol errors in `src/server.rs`
- Result types explicitly specified: `Result<T, AppError>` in client, `Result<CallToolResult, McpError>` in server

**Error Display:**
```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Obsidian API error ({status}): {body}")]
    Api { status: u16, body: String },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
```

## Logging

**Framework:** `tracing` crate with `tracing_subscriber` for initialization

**Patterns:**
- Server startup logs use `tracing::info!()`: `"Starting MCP server with stdio transport"`
- Connection events logged at info level: `"Connecting to Obsidian at {}..."`, `"Connected to Obsidian: {:?}"`
- Errors logged at error level: `tracing::error!("Failed to connect to Obsidian: {}", e)`
- Shutdown events logged at info level: `tracing::info!("Shutting down...")`
- Server configuration includes stderr writer: `.with_writer(std::io::stderr)`

**Filtering:**
- Default filter with module override: `"obsidian_mcp=info"` to log only this crate's messages at info level

## Comments

**When to Comment:**
- Field documentation using `///` for public struct/enum fields with detailed descriptions
- Parameter descriptions in doc comments: `/// Path to the note, e.g. "folder/note.md"`
- Complex logic documented inline with `//` where algorithm is non-obvious
- Helper function purposes documented via `///` when public

**Examples:**
```rust
/// Path to the note, e.g. "folder/note.md"
pub path: String,

/// Delimiter for nested heading paths (default: "::")
pub target_delimiter: Option<String>,

/// Content-Type for the request body. Use "application/json" when setting frontmatter fields to structured values like arrays
pub content_type: Option<String>,
```

**No JSDoc/TSDoc:**
- Rust uses `///` for doc comments (not TSDoc-style)
- Doc comments describe public API, not implementation

## Function Design

**Size:**
- Most tool handler methods 10-25 lines (including error handling)
- Helper methods focused on single responsibility
- `prepare_patch_body` is private helper, 5 lines
- `check_response` is private helper, 5-7 lines with await

**Parameters:**
- Self as `&self` for methods (borrowed reference)
- Async handlers wrap arguments in `Parameters(args)` pattern for macro-driven deserialization
- String references use `&str` for parameters, `String` in struct fields
- Optional parameters use `Option<T>` for true optionality

**Return Values:**
- Handler methods return `Result<CallToolResult, McpError>`
- Client methods return `Result<T, AppError>` where T varies: `String`, `()`, `serde_json::Value`
- Successful results wrapped in `CallToolResult::success(vec![Content::text(...)])`
- Error conversion via `to_mcp_error()` helper for Handler methods

## Module Design

**Exports:**
- `src/lib.rs` re-exports all public modules: `pub mod client`, `pub mod error`, `pub mod server`, `pub mod types`
- Each module is self-contained with clear public API
- Server tool handlers marked with `#[tool]` macro for automatic registration

**Barrel Files:**
- `src/lib.rs` acts as central re-export point for library consumers
- No complex module hierarchies; flat structure preferred

**Visibility:**
- `ObsidianClient` constructor `pub fn new()` is public
- Internal methods like `url()`, `check_response()`, `prepare_patch_body()`, `periodic_url()` are private
- Test modules use `#[cfg(test)]` to gate test-only code

## Documentation Standards

**Struct/Enum Fields:**
- All public fields documented with `///` including examples
- Nested heading path syntax documented for patch operations: `"Heading 1::Subheading 1"`
- Content-Type guidance for frontmatter arrays provided

**Examples:**
```rust
/// Period type: "daily", "weekly", "monthly", "quarterly", "yearly"
pub period: String,

/// Target identifier. For headings: use the heading text without the # prefix.
/// Sub-headings require the full path from the top-level heading using :: as delimiter
/// (e.g. "Heading 1::Subheading 1" to target ## Subheading 1 under # Heading 1).
/// For block references: the block ID. For frontmatter: the field name.
pub target: String,
```

---

*Convention analysis: 2026-03-10*
