# Patch V3 Headers Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Align `patch_note` and `patch_periodic_note` with Obsidian Local REST API v3 by sending the required `Operation`, `Target-Type`, and `Target` headers.

**Architecture:** Introduce `src/types.rs` for shared domain types (`Operation`, `TargetType`, `PatchParams`). Update client methods to accept `&PatchParams` and build HTTP headers from it. Update server arg structs to use the shared enums. Remove the old `X-Heading` / `heading` field.

**Tech Stack:** Rust, serde, schemars (for JSON Schema generation), wiremock (for test mocks), rmcp (MCP protocol SDK).

---

### Task 1: Create `src/types.rs` with shared enums and `PatchParams`

**Files:**
- Create: `src/types.rs`
- Modify: `src/lib.rs:1-3`

**Step 1: Write the failing test**

Add a test in `src/types.rs` that verifies enum serialization and `Display` impl:

```rust
// src/types.rs
use std::fmt;

use schemars::JsonSchema;
use serde::Deserialize;

/// Patch operation to perform on a note target.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Append,
    Prepend,
    Replace,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Append => write!(f, "append"),
            Self::Prepend => write!(f, "prepend"),
            Self::Replace => write!(f, "replace"),
        }
    }
}

/// Type of target within a note to patch.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    Heading,
    Block,
    Frontmatter,
}

impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Heading => write!(f, "heading"),
            Self::Block => write!(f, "block"),
            Self::Frontmatter => write!(f, "frontmatter"),
        }
    }
}

/// Parameters for an Obsidian REST API v3 PATCH request.
#[derive(Debug, Clone)]
pub struct PatchParams {
    pub operation: Operation,
    pub target_type: TargetType,
    pub target: String,
    pub target_delimiter: Option<String>,
    pub trim_target_whitespace: Option<bool>,
    pub create_target_if_missing: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_display() {
        assert_eq!(Operation::Append.to_string(), "append");
        assert_eq!(Operation::Prepend.to_string(), "prepend");
        assert_eq!(Operation::Replace.to_string(), "replace");
    }

    #[test]
    fn target_type_display() {
        assert_eq!(TargetType::Heading.to_string(), "heading");
        assert_eq!(TargetType::Block.to_string(), "block");
        assert_eq!(TargetType::Frontmatter.to_string(), "frontmatter");
    }

    #[test]
    fn operation_deserializes_from_lowercase() {
        let op: Operation = serde_json::from_str("\"append\"").unwrap();
        assert!(matches!(op, Operation::Append));

        let op: Operation = serde_json::from_str("\"prepend\"").unwrap();
        assert!(matches!(op, Operation::Prepend));

        let op: Operation = serde_json::from_str("\"replace\"").unwrap();
        assert!(matches!(op, Operation::Replace));
    }

    #[test]
    fn target_type_deserializes_from_lowercase() {
        let tt: TargetType = serde_json::from_str("\"heading\"").unwrap();
        assert!(matches!(tt, TargetType::Heading));

        let tt: TargetType = serde_json::from_str("\"block\"").unwrap();
        assert!(matches!(tt, TargetType::Block));

        let tt: TargetType = serde_json::from_str("\"frontmatter\"").unwrap();
        assert!(matches!(tt, TargetType::Frontmatter));
    }

    #[test]
    fn invalid_operation_fails_deserialization() {
        let result: Result<Operation, _> = serde_json::from_str("\"invalid\"");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_target_type_fails_deserialization() {
        let result: Result<TargetType, _> = serde_json::from_str("\"invalid\"");
        assert!(result.is_err());
    }
}
```

**Step 2: Register the module in `lib.rs`**

Change `src/lib.rs` to:

```rust
pub mod client;
pub mod error;
pub mod server;
pub mod types;
```

**Step 3: Run tests to verify they pass**

Run: `cargo test --lib types`
Expected: all 5 tests PASS

**Step 4: Commit**

```bash
git add src/types.rs src/lib.rs
git commit -m "feat: add types module with Operation, TargetType, and PatchParams"
```

---

### Task 2: Update `ObsidianClient::patch_note` to use `PatchParams`

**Files:**
- Modify: `src/client.rs:98-117` (method signature + body)
- Modify: `src/client.rs:432-468` (unit tests)

**Step 1: Write the failing test**

Replace the two existing patch_note tests in `src/client.rs` with v3-style tests. Add `use crate::types::{Operation, PatchParams, TargetType};` to the test module's imports:

```rust
#[tokio::test]
async fn patch_note_sends_v3_headers() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/vault/note.md"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "text/markdown"))
        .and(header("Operation", "append"))
        .and(header("Target-Type", "heading"))
        .and(header("Target", "Introduction"))
        .and(body_string("new content"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = mock_client(server.uri());
    let params = PatchParams {
        operation: Operation::Append,
        target_type: TargetType::Heading,
        target: "Introduction".to_string(),
        target_delimiter: None,
        trim_target_whitespace: None,
        create_target_if_missing: None,
    };
    let result = client.patch_note("note.md", &params, "new content").await.unwrap();
    assert_eq!(result, "ok");
}

#[tokio::test]
async fn patch_note_sends_optional_headers() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/vault/note.md"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "text/markdown"))
        .and(header("Operation", "replace"))
        .and(header("Target-Type", "frontmatter"))
        .and(header("Target", "tags"))
        .and(header("Target-Delimiter", "/"))
        .and(header("Trim-Target-Whitespace", "true"))
        .and(header("Create-Target-If-Missing", "true"))
        .and(body_string("new-tag"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = mock_client(server.uri());
    let params = PatchParams {
        operation: Operation::Replace,
        target_type: TargetType::Frontmatter,
        target: "tags".to_string(),
        target_delimiter: Some("/".to_string()),
        trim_target_whitespace: Some(true),
        create_target_if_missing: Some(true),
    };
    let result = client.patch_note("note.md", &params, "new-tag").await.unwrap();
    assert_eq!(result, "ok");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib client::tests::patch_note`
Expected: FAIL — old signature doesn't match

**Step 3: Update the client method**

Replace `src/client.rs:98-117` with:

```rust
pub async fn patch_note(
    &self,
    path: &str,
    params: &PatchParams,
    content: &str,
) -> Result<String, AppError> {
    let mut req = self
        .http
        .patch(self.url(&format!("/vault/{}", path)))
        .header("Authorization", &self.bearer_token)
        .header("Content-Type", "text/markdown")
        .header("Operation", params.operation.to_string())
        .header("Target-Type", params.target_type.to_string())
        .header("Target", &params.target);

    if let Some(ref delimiter) = params.target_delimiter {
        req = req.header("Target-Delimiter", delimiter);
    }
    if let Some(trim) = params.trim_target_whitespace {
        req = req.header("Trim-Target-Whitespace", trim.to_string());
    }
    if let Some(create) = params.create_target_if_missing {
        req = req.header("Create-Target-If-Missing", create.to_string());
    }

    let resp = req.body(content.to_string()).send().await?;
    let resp = self.check_response(resp).await?;
    Ok(resp.text().await?)
}
```

Add the import at the top of `src/client.rs`:

```rust
use crate::types::PatchParams;
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib client::tests::patch_note`
Expected: PASS

**Step 5: Commit**

```bash
git add src/client.rs
git commit -m "feat: update patch_note to send v3 Operation/Target-Type/Target headers"
```

---

### Task 3: Update `ObsidianClient::patch_periodic_note` to use `PatchParams`

**Files:**
- Modify: `src/client.rs:278-300` (method signature + body)
- Modify: `src/client.rs:690-736` (unit tests)

**Step 1: Write the failing test**

Replace the two existing patch_periodic_note tests with:

```rust
#[tokio::test]
async fn patch_periodic_note_sends_v3_headers() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/periodic/daily/2026/3/6/"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "text/markdown"))
        .and(header("Operation", "append"))
        .and(header("Target-Type", "heading"))
        .and(header("Target", "Tasks"))
        .and(body_string("- [ ] do thing"))
        .respond_with(ResponseTemplate::new(200).set_body_string("patched daily"))
        .mount(&server)
        .await;

    let client = mock_client(server.uri());
    let params = PatchParams {
        operation: Operation::Append,
        target_type: TargetType::Heading,
        target: "Tasks".to_string(),
        target_delimiter: None,
        trim_target_whitespace: None,
        create_target_if_missing: None,
    };
    let result = client
        .patch_periodic_note("daily", Some(2026), Some(3), Some(6), &params, "- [ ] do thing")
        .await
        .unwrap();
    assert_eq!(result, "patched daily");
}

#[tokio::test]
async fn patch_periodic_note_without_date() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/periodic/monthly/"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "text/markdown"))
        .and(header("Operation", "replace"))
        .and(header("Target-Type", "block"))
        .and(header("Target", "abc123"))
        .and(body_string("replaced"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = mock_client(server.uri());
    let params = PatchParams {
        operation: Operation::Replace,
        target_type: TargetType::Block,
        target: "abc123".to_string(),
        target_delimiter: None,
        trim_target_whitespace: None,
        create_target_if_missing: None,
    };
    let result = client
        .patch_periodic_note("monthly", None, None, None, &params, "replaced")
        .await
        .unwrap();
    assert_eq!(result, "ok");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib client::tests::patch_periodic`
Expected: FAIL — old signature doesn't match

**Step 3: Update the client method**

Replace `src/client.rs:278-300` with:

```rust
pub async fn patch_periodic_note(
    &self,
    period: &str,
    year: Option<u32>,
    month: Option<u32>,
    day: Option<u32>,
    params: &PatchParams,
    content: &str,
) -> Result<String, AppError> {
    let mut req = self
        .http
        .patch(self.periodic_url(period, year, month, day))
        .header("Authorization", &self.bearer_token)
        .header("Content-Type", "text/markdown")
        .header("Operation", params.operation.to_string())
        .header("Target-Type", params.target_type.to_string())
        .header("Target", &params.target);

    if let Some(ref delimiter) = params.target_delimiter {
        req = req.header("Target-Delimiter", delimiter);
    }
    if let Some(trim) = params.trim_target_whitespace {
        req = req.header("Trim-Target-Whitespace", trim.to_string());
    }
    if let Some(create) = params.create_target_if_missing {
        req = req.header("Create-Target-If-Missing", create.to_string());
    }

    let resp = req.body(content.to_string()).send().await?;
    let resp = self.check_response(resp).await?;
    Ok(resp.text().await?)
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib client::tests::patch_periodic`
Expected: PASS

**Step 5: Commit**

```bash
git add src/client.rs
git commit -m "feat: update patch_periodic_note to send v3 headers"
```

---

### Task 4: Update server arg structs and tool handlers

**Files:**
- Modify: `src/server.rs:46-54` (`PatchNoteArgs`)
- Modify: `src/server.rs:126-140` (`PatchPeriodicNoteArgs`)
- Modify: `src/server.rs:196-207` (`patch_note` handler)
- Modify: `src/server.rs:346-364` (`patch_periodic_note` handler)

**Step 1: Write the failing test**

The e2e tests (Task 5) will verify this end-to-end. For now, just make the compile-time changes. The existing e2e tests will fail to compile — that's the "failing test."

Run: `cargo test --test integration_test`
Expected: FAIL — compile errors because server still passes old args to client methods

**Step 2: Update the arg structs**

Add import at top of `src/server.rs`:

```rust
use crate::types::{Operation, PatchParams, TargetType};
```

Replace `PatchNoteArgs` (`src/server.rs:46-54`):

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PatchNoteArgs {
    /// Path to the note to patch
    pub path: String,
    /// Patch operation: "append", "prepend", or "replace"
    pub operation: Operation,
    /// Target type: "heading", "block", or "frontmatter"
    pub target_type: TargetType,
    /// Target identifier (heading name, block reference ID, or frontmatter field name)
    pub target: String,
    /// Delimiter for nested targets like headings (default: "::")
    pub target_delimiter: Option<String>,
    /// Trim whitespace from target before applying patch
    pub trim_target_whitespace: Option<bool>,
    /// Create the target if it doesn't exist (useful for frontmatter)
    pub create_target_if_missing: Option<bool>,
    /// Content to insert at the target location
    pub content: String,
}
```

Replace `PatchPeriodicNoteArgs` (`src/server.rs:126-140`):

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PatchPeriodicNoteArgs {
    /// Period type: "daily", "weekly", "monthly", "quarterly", "yearly"
    pub period: String,
    /// Year (optional, omit for current period)
    pub year: Option<u32>,
    /// Month (optional)
    pub month: Option<u32>,
    /// Day (optional)
    pub day: Option<u32>,
    /// Patch operation: "append", "prepend", or "replace"
    pub operation: Operation,
    /// Target type: "heading", "block", or "frontmatter"
    pub target_type: TargetType,
    /// Target identifier (heading name, block reference ID, or frontmatter field name)
    pub target: String,
    /// Delimiter for nested targets like headings (default: "::")
    pub target_delimiter: Option<String>,
    /// Trim whitespace from target before applying patch
    pub trim_target_whitespace: Option<bool>,
    /// Create the target if it doesn't exist (useful for frontmatter)
    pub create_target_if_missing: Option<bool>,
    /// Content to insert at the target location
    pub content: String,
}
```

**Step 3: Update the tool handlers**

Replace `patch_note` handler (`src/server.rs:196-207`):

```rust
#[tool(description = "Partially update a note relative to a heading, block reference, or frontmatter field")]
async fn patch_note(
    &self,
    Parameters(args): Parameters<PatchNoteArgs>,
) -> Result<CallToolResult, McpError> {
    let params = PatchParams {
        operation: args.operation,
        target_type: args.target_type,
        target: args.target,
        target_delimiter: args.target_delimiter,
        trim_target_whitespace: args.trim_target_whitespace,
        create_target_if_missing: args.create_target_if_missing,
    };
    let result = self
        .client
        .patch_note(&args.path, &params, &args.content)
        .await
        .map_err(to_mcp_error)?;
    Ok(CallToolResult::success(vec![Content::text(result)]))
}
```

Replace `patch_periodic_note` handler (`src/server.rs:346-364`):

```rust
#[tool(description = "Partially update a periodic note relative to a heading, block reference, or frontmatter field")]
async fn patch_periodic_note(
    &self,
    Parameters(args): Parameters<PatchPeriodicNoteArgs>,
) -> Result<CallToolResult, McpError> {
    let params = PatchParams {
        operation: args.operation,
        target_type: args.target_type,
        target: args.target,
        target_delimiter: args.target_delimiter,
        trim_target_whitespace: args.trim_target_whitespace,
        create_target_if_missing: args.create_target_if_missing,
    };
    let result = self
        .client
        .patch_periodic_note(
            &args.period,
            args.year,
            args.month,
            args.day,
            &params,
            &args.content,
        )
        .await
        .map_err(to_mcp_error)?;
    Ok(CallToolResult::success(vec![Content::text(result)]))
}
```

**Step 4: Run tests to verify compilation succeeds**

Run: `cargo test --lib`
Expected: PASS (all unit tests)

**Step 5: Commit**

```bash
git add src/server.rs
git commit -m "feat: update patch tool arg structs and handlers for v3 API"
```

---

### Task 5: Add e2e tests for patch tools

**Files:**
- Modify: `tests/integration_test.rs` (add 2 new test functions)

**Step 1: Write the e2e tests**

Add these tests to `tests/integration_test.rs`:

```rust
#[tokio::test]
async fn e2e_patch_note() {
    let (mock, client, _cancel) = setup().await;

    Mock::given(method("PATCH"))
        .and(path("/vault/note.md"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "text/markdown"))
        .and(header("Operation", "append"))
        .and(header("Target-Type", "heading"))
        .and(header("Target", "Section 1"))
        .and(body_string("appended text"))
        .respond_with(ResponseTemplate::new(200).set_body_string("done"))
        .mount(&mock)
        .await;

    let result = call_tool(
        &client,
        "patch_note",
        json!({
            "path": "note.md",
            "operation": "append",
            "target_type": "heading",
            "target": "Section 1",
            "content": "appended text"
        }),
    )
    .await;
    assert_eq!(first_text(&result), "done");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn e2e_patch_periodic_note() {
    let (mock, client, _cancel) = setup().await;

    Mock::given(method("PATCH"))
        .and(path("/periodic/daily/"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "text/markdown"))
        .and(header("Operation", "replace"))
        .and(header("Target-Type", "frontmatter"))
        .and(header("Target", "status"))
        .and(header("Create-Target-If-Missing", "true"))
        .and(body_string("done"))
        .respond_with(ResponseTemplate::new(200).set_body_string("updated"))
        .mount(&mock)
        .await;

    let result = call_tool(
        &client,
        "patch_periodic_note",
        json!({
            "period": "daily",
            "operation": "replace",
            "target_type": "frontmatter",
            "target": "status",
            "create_target_if_missing": true,
            "content": "done"
        }),
    )
    .await;
    assert_eq!(first_text(&result), "updated");

    client.cancel().await.unwrap();
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --test integration_test e2e_patch`
Expected: PASS

**Step 3: Run the full test suite**

Run: `cargo make test`
Expected: all tests PASS

**Step 4: Run lint and coverage**

Run: `cargo make check`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add e2e tests for v3 patch_note and patch_periodic_note"
```
