# Server Spec

## Purpose

Defines 16 MCP tools backed by `ObsidianClient` using `FastMCP` `@mcp.tool()` decorators. Defined in `src/obsidian_mcp/server.py`.

## Public Interface

```python
from mcp.server.fastmcp import FastMCP

mcp = FastMCP(
    "obsidian-mcp",
    instructions="MCP server for Obsidian vault operations via Local REST API",
    json_response=True,
)

# Module-level client management
_client: ObsidianClient | None = None
def set_client(client: ObsidianClient) -> None: ...
def get_client() -> ObsidianClient: ...
```

## Tool Inventory

| Tool name | Parameters | Returns | Description |
|-----------|------------|---------|-------------|
| `read_note` | `path: str` | note content string | Read markdown content of a note |
| `create_note` | `path: str, content: str` | confirmation string | Create or overwrite a note |
| `append_note` | `path: str, content: str` | confirmation string | Append content to end of note |
| `patch_note` | `path: str, operation: str, target_type: str, target: str, content: str, ...` | patched content string | Partial update relative to heading/block/frontmatter |
| `delete_note` | `path: str` | confirmation string | Delete a note |
| `list_files` | `path: str \| None = None` | JSON string | List files in a vault directory |
| `search` | `query: str` | JSON string | Full-text search |
| `search_query` | `query: str` | JSON string | Dataview DQL search (TABLE only) |
| `list_commands` | *(none)* | JSON string | List all Obsidian commands |
| `execute_command` | `command_id: str` | confirmation string | Execute an Obsidian command by ID |
| `open_file` | `path: str` | confirmation string | Open a file in Obsidian UI |
| `get_periodic_note` | `period: str, year/month/day: int \| None` | note content string | Read a periodic note |
| `update_periodic_note` | `period: str, content: str, year/month/day: int \| None` | confirmation string | Replace entire periodic note |
| `append_periodic_note` | `period: str, content: str, year/month/day: int \| None` | confirmation string | Append to periodic note |
| `patch_periodic_note` | `period: str, operation: str, target_type: str, target: str, content: str, ...` | patched content string | Partial update periodic note |
| `server_info` | *(none)* | JSON string | Obsidian REST API status and version |

## Tool Parameters

### Shared: periodic note date fields

All periodic note tool functions include:

```python
period: str          # "daily" | "weekly" | "monthly" | "quarterly" | "yearly"
year: int | None     # all three must be non-None for a dated URL; any partial → current period
month: int | None
day: int | None
```

### Shared: patch fields

`patch_note` and `patch_periodic_note` both include:

```python
operation: str                               # "append" | "prepend" | "replace"
target_type: str                             # "heading" | "block" | "frontmatter"
target: str                                  # heading name, block ref ID, or frontmatter key
target_delimiter: str | None = None          # default "::" on Obsidian side
trim_target_whitespace: bool | None = None
create_target_if_missing: bool | None = None
content: str
```

Patch parameters are assembled into a `PatchParams` dataclass via the `_build_patch_params()` helper, which converts `operation` and `target_type` strings to `Operation` and `TargetType` `StrEnum` values.

## Behavior Contracts

### Tool result format

- Success: the tool function returns a `str` value, which is either:
  - A plain string (confirmation messages, markdown content)
  - A JSON string from `json.dumps(result)` (for list/search/info tools)
- Error: exceptions from `ObsidianClient` (`HttpError`, `ApiError`, `JsonError`) propagate naturally; FastMCP converts unhandled exceptions to MCP error responses.

### `search_query` validation

- Validates that `query.lstrip()` starts with `"TABLE"` **before** calling `ObsidianClient`.
- Non-TABLE queries raise `ValueError` immediately.
- Reason: Obsidian Local REST API only supports TABLE DQL queries.

### `get_client()` guard

- Raises `RuntimeError("ObsidianClient not initialized")` if called before `set_client()`.
- Every tool function calls `get_client()` at the top.

### FastMCP configuration

- `name`: `"obsidian-mcp"`
- `instructions`: `"MCP server for Obsidian vault operations via Local REST API"`
- `json_response`: `True`

### Error handling

Exceptions from `ObsidianClient` are not caught in tool functions; they propagate to FastMCP, which converts them to MCP error responses. There is no special-casing of 404 vs 500.

## Invariants

- The `_client` module-level variable must be set via `set_client()` before any tool is invoked.
- All 16 tools are registered at module import time via `@mcp.tool()` decorators; none are optional or feature-flagged.
- `search` and `search_query` accept the same parameter (`query: str`); validation differs between the two.

## Integration Points

- `set_client()` is called from `__main__.py` after constructing `ObsidianClient`.
- Constructs `PatchParams` from tool function parameters via `_build_patch_params()` and delegates to client.
- The `mcp` FastMCP instance is imported and configured in `__main__.py` for transport setup.

## Constraints

- Tool descriptions are static strings in `@mcp.tool()` decorator docstrings; they feed directly into the MCP tool manifest.
- No authentication at the MCP layer; security is delegated to the network (localhost binding).

## Adding a New Tool

1. Add an `async def` function decorated with `@mcp.tool()` in `server.py`, with a descriptive docstring.
2. Use Python type hints for parameters (FastMCP generates the JSON Schema from them).
3. If a new Obsidian API call is needed, add the method to `ObsidianClient` in `client.py` first.
4. Return a `str` on success (plain message or `json.dumps(result)`).
5. Let exceptions propagate naturally; FastMCP handles error conversion.
