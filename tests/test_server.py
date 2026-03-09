"""Tests for MCP server tool definitions."""

import json
from unittest.mock import AsyncMock

import pytest

from obsidian_mcp.client import ServerInfo
from obsidian_mcp.errors import ApiError
from obsidian_mcp.server import (
    append_note,
    append_periodic_note,
    create_note,
    delete_note,
    execute_command,
    get_periodic_note,
    list_commands,
    list_files,
    mcp,
    open_file,
    patch_note,
    patch_periodic_note,
    read_note,
    search,
    search_query,
    server_info,
    set_client,
    update_periodic_note,
)


@pytest.fixture
def mock_client():
    client = AsyncMock()
    set_client(client)
    yield client
    set_client(None)


class TestToolRegistration:
    def test_has_16_tools(self):
        tools = mcp._tool_manager._tools
        assert len(tools) == 16

    def test_tool_names(self):
        tool_names = set(mcp._tool_manager._tools.keys())
        expected = {
            "read_note",
            "create_note",
            "append_note",
            "patch_note",
            "delete_note",
            "list_files",
            "search",
            "search_query",
            "list_commands",
            "execute_command",
            "open_file",
            "get_periodic_note",
            "update_periodic_note",
            "append_periodic_note",
            "patch_periodic_note",
            "server_info",
        }
        assert tool_names == expected


class TestReadNote:
    async def test_returns_content(self, mock_client):
        mock_client.read_note.return_value = "# Hello"
        result = await read_note("note.md")
        assert result == "# Hello"
        mock_client.read_note.assert_awaited_once_with("note.md")


class TestCreateNote:
    async def test_returns_confirmation(self, mock_client):
        result = await create_note("new.md", "content")
        assert result == "Created note at new.md"
        mock_client.create_note.assert_awaited_once_with("new.md", "content")


class TestAppendNote:
    async def test_returns_confirmation(self, mock_client):
        result = await append_note("note.md", "more")
        assert result == "Appended to note.md"
        mock_client.append_note.assert_awaited_once_with("note.md", "more")


class TestPatchNote:
    async def test_returns_patched_content(self, mock_client):
        mock_client.patch_note.return_value = "patched"
        result = await patch_note(
            path="note.md",
            operation="append",
            target_type="heading",
            target="Section",
            content="new text",
        )
        assert result == "patched"
        call_args = mock_client.patch_note.call_args
        assert call_args[0][0] == "note.md"
        params = call_args[0][1]
        assert str(params.operation) == "append"
        assert str(params.target_type) == "heading"
        assert params.target == "Section"


class TestDeleteNote:
    async def test_returns_confirmation(self, mock_client):
        result = await delete_note("old.md")
        assert result == "Deleted old.md"
        mock_client.delete_note.assert_awaited_once_with("old.md")


class TestListFiles:
    async def test_returns_json(self, mock_client):
        mock_client.list_files.return_value = {"files": ["a.md"]}
        result = await list_files()
        assert json.loads(result) == {"files": ["a.md"]}

    async def test_with_path(self, mock_client):
        mock_client.list_files.return_value = {"files": []}
        await list_files("subdir")
        mock_client.list_files.assert_awaited_once_with("subdir")


class TestSearch:
    async def test_returns_json(self, mock_client):
        mock_client.search_simple.return_value = [{"filename": "note.md"}]
        result = await search("query")
        assert json.loads(result) == [{"filename": "note.md"}]


class TestSearchQuery:
    async def test_table_query_succeeds(self, mock_client):
        mock_client.search_query.return_value = [{"file": {"name": "note.md"}}]
        result = await search_query('TABLE file.name FROM ""')
        assert isinstance(json.loads(result), list)

    async def test_non_table_query_raises(self, mock_client):
        with pytest.raises(ValueError, match="Only TABLE queries"):
            await search_query("LIST FROM /")

    async def test_whitespace_before_table(self, mock_client):
        mock_client.search_query.return_value = []
        await search_query("  TABLE file.name")
        mock_client.search_query.assert_awaited_once()


class TestListCommands:
    async def test_returns_json(self, mock_client):
        mock_client.list_commands.return_value = {"commands": []}
        result = await list_commands()
        assert json.loads(result) == {"commands": []}


class TestExecuteCommand:
    async def test_returns_confirmation(self, mock_client):
        result = await execute_command("app:go-back")
        assert result == "Executed command: app:go-back"


class TestOpenFile:
    async def test_returns_confirmation(self, mock_client):
        result = await open_file("note.md")
        assert result == "Opened note.md"


class TestGetPeriodicNote:
    async def test_returns_content(self, mock_client):
        mock_client.get_periodic_note.return_value = "Daily content"
        result = await get_periodic_note("daily")
        assert result == "Daily content"

    async def test_with_date(self, mock_client):
        mock_client.get_periodic_note.return_value = "content"
        await get_periodic_note("daily", 2026, 3, 6)
        mock_client.get_periodic_note.assert_awaited_once_with("daily", 2026, 3, 6)


class TestUpdatePeriodicNote:
    async def test_returns_confirmation(self, mock_client):
        result = await update_periodic_note("weekly", "new content")
        assert result == "Updated weekly periodic note"


class TestAppendPeriodicNote:
    async def test_returns_confirmation(self, mock_client):
        result = await append_periodic_note("daily", "appended")
        assert result == "Appended to daily periodic note"


class TestPatchPeriodicNote:
    async def test_returns_patched_content(self, mock_client):
        mock_client.patch_periodic_note.return_value = "patched"
        result = await patch_periodic_note(
            period="daily",
            operation="append",
            target_type="heading",
            target="Tasks",
            content="- [ ] item",
        )
        assert result == "patched"


class TestServerInfo:
    async def test_returns_json(self, mock_client):
        mock_client.server_info.return_value = ServerInfo(status="OK", versions={"api": "1.0"})
        result = await server_info()
        data = json.loads(result)
        assert data["status"] == "OK"
        assert data["versions"] == {"api": "1.0"}


class TestErrorPropagation:
    async def test_client_error_propagates(self, mock_client):
        mock_client.read_note.side_effect = ApiError(404, "Not found")
        with pytest.raises(ApiError):
            await read_note("missing.md")
