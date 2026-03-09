# Client Spec

## Purpose

`ObsidianClient` wraps `httpx.AsyncClient` to call the Obsidian Local REST API. Each method maps to one HTTP call. Defined in `src/obsidian_mcp/client.py`.

## Public Interface

```python
@dataclass
class ServerInfo:
    status: str
    versions: dict[str, Any]

class ObsidianClient:
    def __init__(self, base_url: str, api_key: str) -> None: ...

    # Async context manager
    async def __aenter__(self) -> ObsidianClient: ...
    async def __aexit__(self, *args: object) -> None: ...

    # Vault operations
    async def read_note(self, path: str) -> str: ...
    async def create_note(self, path: str, content: str) -> None: ...
    async def append_note(self, path: str, content: str) -> None: ...
    async def patch_note(self, path: str, params: PatchParams, content: str) -> str: ...
    async def delete_note(self, path: str) -> None: ...
    async def list_files(self, path: str | None = None) -> Any: ...

    # Search
    async def search_simple(self, query: str) -> Any: ...
    async def search_query(self, query: str) -> Any: ...

    # Commands
    async def list_commands(self) -> Any: ...
    async def execute_command(self, command_id: str) -> None: ...

    # UI
    async def open_file(self, filename: str) -> None: ...

    # Periodic notes
    async def get_periodic_note(self, period: str, year: int | None = None,
                                month: int | None = None, day: int | None = None) -> str: ...
    async def update_periodic_note(self, period: str, year: int | None = None,
                                   month: int | None = None, day: int | None = None,
                                   content: str = "") -> None: ...
    async def append_periodic_note(self, period: str, year: int | None = None,
                                   month: int | None = None, day: int | None = None,
                                   content: str = "") -> None: ...
    async def patch_periodic_note(self, period: str, year: int | None = None,
                                  month: int | None = None, day: int | None = None,
                                  params: PatchParams | None = None,
                                  content: str = "") -> str: ...

    # Health
    async def server_info(self) -> ServerInfo: ...
```

## HTTP Method Mapping

| Method | HTTP Verb | Obsidian endpoint pattern |
|--------|-----------|--------------------------|
| `read_note` | GET | `/vault/{path}` |
| `create_note` | PUT | `/vault/{path}` |
| `append_note` | POST | `/vault/{path}` |
| `patch_note` | PATCH | `/vault/{path}` |
| `delete_note` | DELETE | `/vault/{path}` |
| `list_files` | GET | `/vault/` or `/vault/{path}/` |
| `search_simple` | POST | `/search/simple/` |
| `search_query` | POST | `/search/` |
| `list_commands` | GET | `/commands/` |
| `execute_command` | POST | `/commands/{id}/` |
| `open_file` | POST | `/open/{filename}` |
| `get_periodic_note` | GET | `/periodic/{period}/` or `/periodic/{period}/{y}/{m}/{d}/` |
| `update_periodic_note` | PUT | (same URL pattern as get) |
| `append_periodic_note` | POST | (same URL pattern as get) |
| `patch_periodic_note` | PATCH | (same URL pattern as get) |
| `server_info` | GET | `/` |

## Behavior Contracts

### `__init__(base_url, api_key)`

- Builds an `httpx.AsyncClient` with `verify=False` (Obsidian uses a self-signed cert).
- Pre-formats `_bearer_token` as `"Bearer {api_key}"` at construction time.
- Sets the `Authorization` header on the `httpx.AsyncClient` instance directly.

### Async context manager

- `__aenter__` returns `self`.
- `__aexit__` calls `self._client.aclose()` to cleanly shut down the HTTP client.

### `_check_response(resp)`

- If `resp.is_success` → returns `resp`.
- Otherwise → reads body text and raises `ApiError(status_code, body)`.
- All methods call this helper before consuming the response body.

### Error wrapping

- `httpx.HTTPError` is caught and re-raised as `HttpError(e)`.
- `ValueError` (from `resp.json()`) is caught and re-raised as `JsonError(e)`.
- `ApiError` is raised directly by `_check_response()`.

### Vault notes

- `read_note`: sends `Accept: text/markdown`; returns raw markdown string.
- `create_note`: sends `Content-Type: text/markdown`; uses PUT (idempotent, overwrites if exists).
- `append_note`: sends `Content-Type: text/markdown`; uses POST (appends to end).
- `patch_note`: sends `Content-Type: text/markdown`; PATCH params transmitted as HTTP headers (not body):
  - `Operation: {append|prepend|replace}` (required)
  - `Target-Type: {heading|block|frontmatter}` (required)
  - `Target: {identifier}` (required)
  - `Target-Delimiter: {string}` (optional, omitted if `None`)
  - `Trim-Target-Whitespace: {true|false}` (optional, omitted if `None`)
  - `Create-Target-If-Missing: {true|false}` (optional, omitted if `None`)
  - Returns the patched content as a string.
- `delete_note`: no body; returns `None`.
- `list_files(None)` → `/vault/`; `list_files("dir")` → `/vault/dir/` (trailing slash required).

### Search

- `search_simple`: query sent as URL query param `?query={value}`.
- `search_query`: query sent as request body with `Content-Type: application/vnd.olrapi.dataview.dql+txt`.

### Periodic notes

- `_periodic_url(period, year, month, day)`:
  - All three of `year`, `month`, `day` must be non-`None` to produce `/periodic/{period}/{y}/{m}/{d}/`.
  - Any partial combination (e.g. year only, year+month) falls back to `/periodic/{period}/` (current period).
  - Valid period values: `"daily"`, `"weekly"`, `"monthly"`, `"quarterly"`, `"yearly"`.
- `patch_periodic_note`: raises `ValueError` if `params` is `None`.

## Invariants

- All requests include `Authorization: Bearer {token}` header (set on the `httpx.AsyncClient` instance).
- TLS certificate validation is always disabled; the client is designed for local Obsidian only.
- `patch_note` and `patch_periodic_note` pass parameters via HTTP headers, not JSON body.
- Partial date parameters always fall back to current period; there is no partial-date URL form.

## Integration Points

- Constructed in `__main__.py`; used as an async context manager.
- Set on the `server` module via `set_client()` for tool functions to access via `get_client()`.
- `AppError` (and subclasses `HttpError`, `ApiError`, `JsonError`) raised by all methods propagate as exceptions through the MCP tool functions.
- `PatchParams` from `types.py` drives header construction in `patch_note` / `patch_periodic_note`.

## Constraints

- Designed for local use only (self-signed cert acceptance, localhost default).
- No retry logic; each method makes exactly one HTTP request.
- `list_files` returns raw `Any` (parsed JSON); the schema mirrors Obsidian's `{"files": [...]}` response but is not typed.
