# E2E Testing Prerequisites

The e2e integration tests run against a real Obsidian instance via the Local REST API.
They exercise the full MCP stack: MCP tool → ObsidianClient → Obsidian REST API.

## Requirements

1. **Obsidian** running on the macOS host with a vault open
2. **Local REST API plugin** installed and enabled in Obsidian (default port: 27124)
3. **Network access** from the test environment to the Obsidian host
   - From OrbStack containers: `https://host.orb.internal:27124`

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `OBSIDIAN_API_KEY` | Yes | API key from Obsidian Local REST API plugin settings |
| `OBSIDIAN_API_URL` | No | Obsidian REST API URL (default: `https://127.0.0.1:27124`) |

If `OBSIDIAN_API_KEY` is not set, all e2e tests are **skipped** (not failed).

## Running

```bash
# On macOS host (default URL works)
OBSIDIAN_API_KEY="your-api-key-here" just e2e

# In OrbStack container (override URL to reach host)
OBSIDIAN_API_URL=https://host.orb.internal:27124 OBSIDIAN_API_KEY="your-api-key-here" just e2e
```

## Test Isolation

- All write operations are scoped to the `tests/` folder inside the vault.
- Each test cleans up the `tests/` folder before and after execution.
- Periodic note tests modify the current period's note and restore it after.

## Troubleshooting

- **Tests skip silently:** Check that `OBSIDIAN_API_KEY` is set in the command.
- **Connection refused:** Ensure Obsidian is running and the Local REST API plugin is enabled.
- **TLS errors:** The test client accepts self-signed certificates (Obsidian uses self-signed TLS).
