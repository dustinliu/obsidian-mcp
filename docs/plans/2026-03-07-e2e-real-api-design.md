# Design: E2E Tests Against Real Obsidian Local REST API

**Date:** 2026-03-07

## Goal

Replace the wiremock-based e2e tests with tests that run against the real Obsidian Local REST API, providing true end-to-end validation of the full MCP stack.

## Architecture

```
Test process (OrbStack container)
  MCP client
    → Axum HTTP server (in-process)
      → ObsidianServer (MCP handler)
        → ObsidianClient (reqwest)
          → Real Obsidian Local REST API (https://host.orb.internal:27124)
```

The test setup is identical to the current wiremock-based approach, except `ObsidianClient` points to the real Obsidian API on the macOS host instead of a wiremock mock server.

## Configuration

- **Host URL:** `https://host.orb.internal:27124` (hardcoded — fixed for OrbStack containers)
- **API key:** Read from `OBSIDIAN_API_KEY` environment variable. Tests fail with a clear message if not set.

## Test Isolation

- All write operations use the `tests/` folder inside the vault.
- Each test cleans up the `tests/` folder at the start (to ensure a clean state even if a previous run crashed) and again at the end.
- Read-only operations on vault-level endpoints (e.g. `list_files`, `search`, `server_info`, `list_commands`) do not need folder isolation.

## Tool Coverage

All 16 tools are tested:

### Data tools (verify request and response content)
1. `read_note` — create a note, read it back, verify content
2. `create_note` — create a note, verify success message
3. `append_note` — create a note, append to it, read back and verify
4. `patch_note` — create a note with headings, patch a section, read back and verify
5. `delete_note` — create a note, delete it, verify it's gone
6. `list_files` — list files, verify response structure
7. `search` — create a note with known content, search for it
8. `search_query` — run a Dataview DQL query
9. `get_periodic_note` — read a periodic note
10. `update_periodic_note` — update a periodic note, read back and verify
11. `append_periodic_note` — append to a periodic note, read back and verify
12. `patch_periodic_note` — patch a section of a periodic note
13. `server_info` — verify response contains status/version info

### UI tools (verify API call succeeds, no result assertion)
14. `list_commands` — verify response is valid JSON array
15. `execute_command` — execute a safe command, verify no error
16. `open_file` — open a test file, verify no error

## Changes

1. **Rewrite `tests/integration_test.rs`** — replace wiremock setup with real API connection
2. **Remove `wiremock` dependency** from `Cargo.toml` (dev-dependencies)
3. **Create `docs/e2e-testing.md`** — document prerequisites for running e2e tests
4. **Update `CLAUDE.md`** — reference the new e2e testing doc

## Test Structure

```rust
// setup() creates:
// - ObsidianClient pointing to real API
// - MCP server + client (same Axum wiring as before)
// - Cleans up tests/ folder

// cleanup() deletes all notes in tests/ folder

// Each test:
// 1. Calls cleanup (defensive)
// 2. Creates any needed fixture notes via MCP tools
// 3. Exercises the tool under test
// 4. Asserts results
// 5. Calls cleanup
```

## Risks

- Tests depend on Obsidian running on the macOS host with Local REST API plugin enabled.
- Periodic note tests depend on the Periodic Notes plugin configuration.
- Network flakiness between container and host could cause intermittent failures.
- Tests mutate real vault state (mitigated by `tests/` folder isolation + cleanup).
