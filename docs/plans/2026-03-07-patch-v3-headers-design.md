# Design: Align PATCH tools with Obsidian Local REST API v3

**Date:** 2026-03-07
**Status:** Approved

## Problem

The `patch_note` and `patch_periodic_note` MCP tools send an `X-Heading` header (v2 API). The Obsidian Local REST API v3 replaced this with three required headers: `Operation`, `Target-Type`, and `Target`. The current implementation silently fails or produces incorrect results.

## Solution: Approach A — Direct v3 alignment

Update both tools and their underlying client methods to send the correct v3 headers. No backward-compatibility shims since the old API was non-functional.

## New shared module: `src/types.rs`

Introduce `types.rs` for domain types shared between client and server. Dependency chain:

```
server -> client -> error
  \         \
   -> types <-
```

### Types

```rust
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Append,
    Prepend,
    Replace,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    Heading,
    Block,
    Frontmatter,
}

#[derive(Debug, Clone)]
pub struct PatchParams {
    pub operation: Operation,
    pub target_type: TargetType,
    pub target: String,
    pub target_delimiter: Option<String>,
    pub trim_target_whitespace: Option<bool>,
    pub create_target_if_missing: Option<bool>,
}
```

Both enums implement `Display` for HTTP header value serialization.

## MCP tool args changes

**Remove** from both `PatchNoteArgs` and `PatchPeriodicNoteArgs`:
- `heading: Option<String>`

**Add** to both:
- `operation: Operation` (required)
- `target_type: TargetType` (required)
- `target: String` (required)
- `target_delimiter: Option<String>`
- `trim_target_whitespace: Option<bool>`
- `create_target_if_missing: Option<bool>`

## Client method changes

Both `patch_note` and `patch_periodic_note` change signature to accept `&PatchParams` instead of `heading: Option<&str>`. The methods construct HTTP headers from `PatchParams`:

- `Operation` header from `params.operation`
- `Target-Type` header from `params.target_type`
- `Target` header from `params.target`
- Optional headers only sent when `Some`

Remove `X-Heading` header entirely.

## Error handling

No new error variants needed. Invalid enum values are rejected at serde deserialization. HTTP errors handled by existing `check_response()`.

## Testing

- **Unit tests (client.rs):** wiremock assertions that correct headers are sent for each target type
- **Unit tests (server.rs):** args deserialization with new fields
- **E2e tests (integration_test.rs):** full MCP client -> server -> wiremock flow
