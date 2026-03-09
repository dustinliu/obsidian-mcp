# Types Spec

## Purpose

Defines shared domain types used across `client.py` and `server.py`. Defined in `src/obsidian_mcp/types.py` (domain types) and `src/obsidian_mcp/errors.py` (error types).

## Public Interface

### `Operation` (`types.py`)

```python
class Operation(StrEnum):
    """Patch operation type."""
    APPEND = "append"
    PREPEND = "prepend"
    REPLACE = "replace"
```

### `TargetType` (`types.py`)

```python
class TargetType(StrEnum):
    """Patch target type."""
    HEADING = "heading"
    BLOCK = "block"
    FRONTMATTER = "frontmatter"
```

### `PatchParams` (`types.py`)

```python
@dataclass
class PatchParams:
    """Parameters for an Obsidian REST API v3 PATCH request."""
    operation: Operation
    target_type: TargetType
    target: str
    target_delimiter: str | None = None
    trim_target_whitespace: bool | None = None
    create_target_if_missing: bool | None = None
```

### Error types (`errors.py`)

```python
class AppError(Exception):
    """Base exception for all Obsidian MCP errors."""

class HttpError(AppError):
    """Wraps httpx transport errors (connection, timeout, TLS)."""
    def __init__(self, error: Exception) -> None: ...
    # self.error: Exception

class ApiError(AppError):
    """Non-2xx HTTP response from the Obsidian API."""
    def __init__(self, status: int, body: str) -> None: ...
    # self.status: int
    # self.body: str

class JsonError(AppError):
    """JSON deserialization failure."""
    def __init__(self, error: Exception) -> None: ...
    # self.error: Exception
```

## Behavior Contracts

### `Operation` / `TargetType`

- `str()` output is the lowercase variant value (inherited from `StrEnum`); used verbatim as HTTP header values in PATCH requests.
- Construction from a string (e.g. `Operation("append")`) only accepts the exact lowercase values; any other value raises `ValueError`.

### `PatchParams`

- Carries all parameters for an Obsidian REST API v3 PATCH request.
- Optional fields map to optional HTTP headers; absent fields (`None`) mean the header is omitted entirely.
- `target_delimiter`: defaults to `"::"` on the Obsidian side when omitted.

### Error types

- `HttpError`: wraps `httpx.HTTPError` for connection/timeout/TLS failures.
- `ApiError`: created by `ObsidianClient._check_response()` for any non-2xx status; `body` is the raw response text.
- `JsonError`: wraps `ValueError` from `resp.json()` for response deserialization failures.
- Display formats (via `str()`):
  - `HttpError` → `"HTTP request failed: {error}"`
  - `ApiError`  → `"Obsidian API error ({status}): {body}"`
  - `JsonError` → `"JSON error: {error}"`
- All three inherit from `AppError`, which inherits from `Exception`.

## Invariants

- `Operation` and `TargetType` are the exhaustive sets; adding a new variant requires updating all match/dispatch logic and the Obsidian API docs must support the new value.
- The error hierarchy (`AppError` → `HttpError`/`ApiError`/`JsonError`) covers all failure modes of `ObsidianClient`. The variants are mutually exclusive.

## Integration Points

- `PatchParams` is constructed in `server.py` via `_build_patch_params()` from tool function parameters and passed to `ObsidianClient.patch_note()` / `patch_periodic_note()`.
- Error types are raised by `ObsidianClient` methods and propagate through MCP tool functions; FastMCP converts unhandled exceptions to MCP error responses.
