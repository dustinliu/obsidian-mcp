# Codebase Concerns

**Analysis Date:** 2026-03-10

## Security Considerations

**Invalid TLS Certificate Acceptance:**
- Risk: The HTTP client in `ObsidianClient::new()` deliberately accepts invalid/self-signed TLS certificates using `.danger_accept_invalid_certs(true)` (line 23 in `src/client.rs`). This is necessary for local Obsidian instances but creates a potential attack vector in production or compromised local networks.
- Files: `src/client.rs:23`
- Current mitigation: This is intentional by design—Obsidian Local REST API uses self-signed certs. Documented in CLAUDE.md.
- Recommendations:
  - Add a CLI flag `--insecure-tls` or environment variable to control this behavior
  - Document the security implications in README
  - Consider certificate pinning as an option for production use

**API Key Exposure in Logs:**
- Risk: API keys are passed as plain strings and used in Bearer tokens. While not logged directly, error messages containing API responses could leak sensitive information from vault content
- Files: `src/main.rs:67-73`, `src/client.rs:27`
- Current mitigation: Error messages use structured logging to stderr, not stdout
- Recommendations: Sanitize error messages to avoid logging full API response bodies for sensitive operations

## Panic Points (Runtime Failures)

**HTTP Client Construction Panic:**
- Issue: `expect("failed to build HTTP client")` at `src/client.rs:25` will panic if the HTTP client cannot be built
- Files: `src/client.rs:21-34`
- Impact: Prevents server startup without clear error handling to caller
- Fix approach: Return `Result` from `new()` method or use `.map_err()` to convert to a recoverable error. Current caller in `main.rs:64` ignores this risk.

**Test Panic in Error Handling:**
- Issue: Unit test helper `text_content()` at `src/server.rs:479-483` panics with `panic!("expected text content")` if content is not text
- Files: `src/server.rs:479-483`
- Impact: Tests fail with unhelpful panic message instead of clear assertion failure
- Fix approach: Replace `panic!` with `.expect()` or proper test assertion

**Integration Test Helper Panic:**
- Issue: `first_text()` at `tests/integration_test.rs:144-153` panics with `.expect("expected text content in tool result")` if no text content found
- Files: `tests/integration_test.rs:144-153`
- Impact: E2E test failures produce opaque panics instead of diagnostic messages
- Fix approach: Return `Result` type and use `?` operator, or provide clearer assertion messages

## Fragile Areas

**Partial Date Parameter Handling:**
- Issue: Periodic note functions accept `(year, month, day)` as separate optional parameters. The logic in `periodic_url()` at `src/client.rs:218-231` requires ALL three to be present; partial date combinations silently fall back to period-only URL
- Files: `src/client.rs:218-231`, `src/server.rs:354-365` (GetPeriodicNoteArgs) and equivalents
- Why fragile: This creates unexpected behavior—caller might pass year+month expecting a specific date but gets the current period instead. Test at `src/client.rs:384-393` documents this, but it's not validated in tool handlers.
- Safe modification: Add validation in tool handlers to reject partial date combinations with clear error messages
- Test coverage: Covered by unit tests for `periodic_url()` but tool-layer tests only use either all dates or none

**Heading Path Delimiter Hardcoded**:
- Issue: Sub-heading paths use `::` delimiter but there's no validation that user targets don't contain `::`
- Files: `src/server.rs:56-57` (documentation in PatchNoteArgs)
- Why fragile: A heading literally containing `::` would be incorrectly parsed as nested hierarchy
- Safe modification: Escape or validate heading targets before passing to API
- Test coverage: No test for headings containing the delimiter

**Serialization Type Mismatch for Frontmatter:**
- Issue: `PatchParams.content_type` defaults to `text/markdown` but must be explicitly set to `application/json` for structured data like arrays (documented in `src/server.rs:66-67`)
- Files: `src/server.rs:66-67`, `src/client.rs:105`, `src/client.rs:300`
- Why fragile: Users can pass JSON arrays without setting `content_type` and get unexpected results; no validation prevents this
- Safe modification: Add validation to require `application/json` content_type when body appears to be JSON
- Test coverage: Tested with valid JSON in `src/client.rs:534-562` but no test for mismatched content-type

## Error Handling Gaps

**API Error Body Loss on Exception:**
- Issue: In `check_response()` at `src/client.rs:40-48`, if `resp.text().await` itself fails, the error details are lost and `.unwrap_or_default()` returns an empty string
- Files: `src/client.rs:40-48`
- Impact: Users see "Obsidian API error (500): " with no body explanation
- Fix approach: Chain error handling or log intermediate failures

**Incomplete Error Context in Network Failures:**
- Issue: `AppError::Http` variant (from `reqwest::Error`) may not include URL or method info that could help debugging
- Files: `src/error.rs:5-6`
- Impact: Difficult to diagnose which API call failed when there are multiple concurrent requests
- Fix approach: Wrap reqwest errors with context (URL, method, attempt count)

## Test Coverage Gaps

**E2E Test Cleanup Fragility:**
- Issue: Integration tests in `tests/integration_test.rs` clean up test files but `cleanup()` at line 102-113 silently ignores deletion errors
- Files: `tests/integration_test.rs:102-113`
- Impact: Stale test files accumulate in vault if deletion fails, breaking future test isolation
- Risk: Medium—tests may pass locally but fail in CI if cleanup races with other operations
- Priority: Medium

**No Test for Command Execution Side Effects:**
- Issue: `execute_command()` tool allows arbitrary Obsidian command execution (e.g., `editor:toggle-bold`) but no test validates that side effects don't corrupt vault state
- Files: `src/server.rs:324-337`, `tests/integration_test.rs:462-482`
- Impact: Dangerous commands could be executed without warning. E2E test only validates API call success, not safety.
- Risk: Low if users are trusted, but HIGH if exposed to untrusted clients
- Priority: High if moving to multi-user or network deployment

**No Test for Concurrent Access:**
- Issue: Single `ObsidianClient` is shared across multiple MCP handlers (wrapped in `Arc`) but no test validates thread safety or handles race conditions
- Files: `src/main.rs:64`, `src/server.rs:18-19`
- Impact: Unknown behavior under concurrent tool invocations
- Priority: Medium—important before production use

**Missing Test for Large Content:**
- Issue: No test for reading/writing very large notes (>100MB). Streaming behavior is unknown.
- Files: `src/client.rs:70`, `src/client.rs:85-97`
- Impact: Unknown memory usage patterns and potential OOM scenarios
- Priority: Low but worth documenting limitations

**No Validation Test for Malformed Input:**
- Issue: Limited testing for edge cases like empty paths, null bytes in content, extremely long heading targets
- Files: All tool handlers in `src/server.rs`
- Impact: May crash or behave unexpectedly with invalid input
- Priority: Medium

## Known Bugs

**Newline Handling in Append Operations Inconsistent:**
- Symptoms: `prepare_patch_body()` at `src/client.rs:328-333` adds `\n` only for Append operations. Prepend and Replace don't get auto-newline.
- Files: `src/client.rs:328-333`
- Trigger: Call `patch_note` with `operation: Prepend` and content without leading newline; result may concatenate incorrectly
- Workaround: Always include leading/trailing newlines in content for Prepend/Replace operations
- Fix: Clarify and document whether the API or client is responsible for newline normalization

## Scaling Limits

**Single HTTP Client Shared Across All Tools:**
- Current capacity: The shared `reqwest::Client` wrapped in `Arc<ObsidianClient>` handles all concurrent requests with tokio's thread pool
- Limit: Large numbers of concurrent operations (>1000) may exhaust connection pools or tokio tasks
- Scaling path: Add connection pool configuration, implement request queuing, or switch to bounded concurrency

**No Request Timeout:**
- Issue: HTTP requests in `ObsidianClient` methods have no explicit timeout configuration
- Files: All methods in `src/client.rs`
- Impact: Slow Obsidian instances or network issues could cause indefinite hangs
- Fix: Set `timeout()` on the `Client` builder

**No Rate Limiting:**
- Issue: Multiple concurrent API calls to Obsidian could overwhelm the local instance if it has limited resources
- Files: All HTTP methods in `src/client.rs`
- Impact: Denial-of-service risk from internal tool abuse
- Fix: Implement per-client or global rate limiter

## Dependencies at Risk

**rmcp Version Lock:**
- Risk: Locked to `rmcp 0.12` with no patch range specified. If a critical security fix is released in `0.12.x`, it won't auto-update
- Impact: Security vulnerabilities could go unfixed
- Migration plan: Use `0.12.*` range in Cargo.toml or implement regular dependency audits

**Cargo Edition 2024:**
- Risk: Edition is set to `2024` in `Cargo.toml:4` which is likely a typo (should be `2021`). This may prevent building on stable toolchain.
- Files: `Cargo.toml:4`
- Impact: Build failures on standard Rust environments
- Fix: Change to valid edition `"2021"`

## Technical Debt

**Argument Struct Duplication:**
- Issue: Multiple arg structs for different tools have nearly identical structure (e.g., `PatchNoteArgs` and `PatchPeriodicNoteArgs` are identical except for period field)
- Files: `src/server.rs:48-71`, `src/server.rs:142-171`
- Impact: Code duplication makes refactoring error-prone
- Fix approach: Use a single `PatchArgs` struct with optional period, or create a macro to reduce repetition

**Duplicated Patch Logic in Client:**
- Issue: `patch_note()` and `patch_periodic_note()` in `src/client.rs` have nearly identical implementations (lines 99-131 and 291-326)
- Files: `src/client.rs:99-131`, `src/client.rs:291-326`
- Impact: Bug fixes in one require duplication to the other
- Fix approach: Extract common logic into a `fn patch_impl()` helper

**Client Helper Duplication:**
- Issue: `send()` -> `check_response()` -> extract body pattern is repeated in 14 methods
- Files: `src/client.rs` multiple methods
- Impact: Each method is >10 lines; harder to audit for consistency
- Fix approach: Create helper `send_and_check()` that returns text/json directly

**String URL Building:**
- Issue: Path construction via string formatting (e.g., `format!("/vault/{}", path)`) is error-prone and not type-safe
- Files: `src/client.rs` throughout
- Impact: Path traversal vulnerabilities if user input isn't sanitized (likely OK since Obsidian API validates)
- Fix approach: Use a Path builder or at minimum add validation that path doesn't contain `..` or `//`

## Documentation Gaps

**No Comment on TLS Acceptance Decision:**
- Issue: `.danger_accept_invalid_certs(true)` appears without explanation
- Files: `src/client.rs:23`
- Fix: Add inline comment explaining why this is necessary (Obsidian Local REST API uses self-signed certs)

**Periodic URL Fallback Behavior Undocumented:**
- Issue: Callers of `get_periodic_note()` may not realize partial date params silently fallback to period-only
- Files: `src/client.rs:233-249` and tool handlers
- Fix: Document this in tool descriptions or add explicit validation

**No Maximum Content Size Documentation:**
- Issue: Unknown if there are limits on note size, search query size, etc.
- Files: API boundary (client.rs)
- Fix: Add documentation or tests to establish and document limits

---

*Concerns audit: 2026-03-10*
