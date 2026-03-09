"""MCP server with 16 Obsidian vault tools."""

import json

from mcp.server.fastmcp import FastMCP

from obsidian_mcp.client import ObsidianClient
from obsidian_mcp.types import Operation, PatchParams, TargetType

mcp = FastMCP(
    "obsidian-mcp",
    instructions="MCP server for Obsidian vault operations via Local REST API",
    json_response=True,
)

# Client is set at startup before tools are called
_client: ObsidianClient | None = None


def set_client(client: ObsidianClient) -> None:
    global _client
    _client = client


def get_client() -> ObsidianClient:
    if _client is None:
        raise RuntimeError("ObsidianClient not initialized")
    return _client


def _build_patch_params(
    operation: str,
    target_type: str,
    target: str,
    target_delimiter: str | None = None,
    trim_target_whitespace: bool | None = None,
    create_target_if_missing: bool | None = None,
) -> PatchParams:
    return PatchParams(
        operation=Operation(operation),
        target_type=TargetType(target_type),
        target=target,
        target_delimiter=target_delimiter,
        trim_target_whitespace=trim_target_whitespace,
        create_target_if_missing=create_target_if_missing,
    )


# --- Vault notes ---


@mcp.tool()
async def read_note(path: str) -> str:
    """Read the content of a note at the given path."""
    client = get_client()
    return await client.read_note(path)


@mcp.tool()
async def create_note(path: str, content: str) -> str:
    """Create a new note or overwrite an existing one."""
    client = get_client()
    await client.create_note(path, content)
    return f"Created note at {path}"


@mcp.tool()
async def append_note(path: str, content: str) -> str:
    """Append content to the end of an existing note."""
    client = get_client()
    await client.append_note(path, content)
    return f"Appended to {path}"


@mcp.tool()
async def patch_note(
    path: str,
    operation: str,
    target_type: str,
    target: str,
    content: str,
    target_delimiter: str | None = None,
    trim_target_whitespace: bool | None = None,
    create_target_if_missing: bool | None = None,
) -> str:
    """Partially update a note relative to a heading, block reference, or frontmatter field."""
    client = get_client()
    params = _build_patch_params(
        operation,
        target_type,
        target,
        target_delimiter,
        trim_target_whitespace,
        create_target_if_missing,
    )
    return await client.patch_note(path, params, content)


@mcp.tool()
async def delete_note(path: str) -> str:
    """Delete a note from the vault."""
    client = get_client()
    await client.delete_note(path)
    return f"Deleted {path}"


@mcp.tool()
async def list_files(path: str | None = None) -> str:
    """List files in a vault directory."""
    client = get_client()
    result = await client.list_files(path)
    return json.dumps(result)


# --- Search ---


@mcp.tool()
async def search(query: str) -> str:
    """Search notes by text query."""
    client = get_client()
    result = await client.search_simple(query)
    return json.dumps(result)


@mcp.tool()
async def search_query(query: str) -> str:
    """Search notes using Dataview DQL query.

    Only TABLE queries are supported (e.g. 'TABLE file.ctime FROM "folder"').
    LIST and TASK query types are not supported by the Obsidian Local REST API.
    """
    if not query.lstrip().startswith("TABLE"):
        raise ValueError(
            "Only TABLE queries are supported. "
            "LIST and TASK query types are not supported by the Obsidian Local REST API."
        )
    client = get_client()
    result = await client.search_query(query)
    return json.dumps(result)


# --- Commands ---


@mcp.tool()
async def list_commands() -> str:
    """List all available Obsidian commands."""
    client = get_client()
    result = await client.list_commands()
    return json.dumps(result)


@mcp.tool()
async def execute_command(command_id: str) -> str:
    """Execute an Obsidian command by its ID."""
    client = get_client()
    await client.execute_command(command_id)
    return f"Executed command: {command_id}"


# --- UI ---


@mcp.tool()
async def open_file(path: str) -> str:
    """Open a file in the Obsidian user interface."""
    client = get_client()
    await client.open_file(path)
    return f"Opened {path}"


# --- Periodic notes ---


@mcp.tool()
async def get_periodic_note(
    period: str,
    year: int | None = None,
    month: int | None = None,
    day: int | None = None,
) -> str:
    """Read a periodic note (daily, weekly, monthly, quarterly, yearly)."""
    client = get_client()
    return await client.get_periodic_note(period, year, month, day)


@mcp.tool()
async def update_periodic_note(
    period: str,
    content: str,
    year: int | None = None,
    month: int | None = None,
    day: int | None = None,
) -> str:
    """Replace the entire content of a periodic note."""
    client = get_client()
    await client.update_periodic_note(period, year, month, day, content)
    return f"Updated {period} periodic note"


@mcp.tool()
async def append_periodic_note(
    period: str,
    content: str,
    year: int | None = None,
    month: int | None = None,
    day: int | None = None,
) -> str:
    """Append content to a periodic note."""
    client = get_client()
    await client.append_periodic_note(period, year, month, day, content)
    return f"Appended to {period} periodic note"


@mcp.tool()
async def patch_periodic_note(
    period: str,
    operation: str,
    target_type: str,
    target: str,
    content: str,
    year: int | None = None,
    month: int | None = None,
    day: int | None = None,
    target_delimiter: str | None = None,
    trim_target_whitespace: bool | None = None,
    create_target_if_missing: bool | None = None,
) -> str:
    """Partially update a periodic note relative to a heading, block reference, or frontmatter."""
    client = get_client()
    params = _build_patch_params(
        operation,
        target_type,
        target,
        target_delimiter,
        trim_target_whitespace,
        create_target_if_missing,
    )
    return await client.patch_periodic_note(
        period, year, month, day, params=params, content=content
    )


# --- Health ---


@mcp.tool()
async def server_info() -> str:
    """Get Obsidian Local REST API server status and version info."""
    client = get_client()
    info = await client.server_info()
    return json.dumps({"status": info.status, "versions": info.versions})
