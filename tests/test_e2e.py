"""E2E tests against a real Obsidian instance via MCP.

Requires OBSIDIAN_API_KEY env var. Tests are skipped if not set.
All write operations are scoped to the tests/ folder inside the vault.
"""

import json
import os

import pytest
from dotenv import load_dotenv

load_dotenv()

from obsidian_mcp.client import ObsidianClient  # noqa: E402
from obsidian_mcp.server import mcp, set_client  # noqa: E402

# Skip all tests if no API key
pytestmark = pytest.mark.skipif(
    not os.environ.get("OBSIDIAN_API_KEY"),
    reason="OBSIDIAN_API_KEY not set",
)

API_KEY = os.environ.get("OBSIDIAN_API_KEY", "")
API_URL = os.environ.get("OBSIDIAN_API_URL", "https://127.0.0.1:27124")


@pytest.fixture(scope="session")
def anyio_backend():
    return "asyncio"


@pytest.fixture(scope="session")
async def client():
    """Create a shared ObsidianClient for all e2e tests."""
    c = ObsidianClient(API_URL, API_KEY)
    set_client(c)
    yield c
    try:
        await c._client.aclose()
    except RuntimeError:
        pass
    set_client(None)


@pytest.fixture(autouse=True)
async def cleanup(client):
    """Clean up test files before and after each test."""
    await _cleanup_test_files(client)
    yield
    await _cleanup_test_files(client)


async def _cleanup_test_files(client: ObsidianClient):
    """Delete all files in the tests/ folder."""
    try:
        result = await client.list_files("tests")
        files = result.get("files", [])
        for f in files:
            try:
                await client.delete_note(f)
            except Exception:
                pass
    except Exception:
        pass


async def _call_tool(tool_name: str, **kwargs) -> str:
    """Call an MCP tool by name and return its text result."""
    tools = mcp._tool_manager._tools
    tool = tools[tool_name]
    result = await tool.run(kwargs)
    return result


# --- Tool listing ---


class TestListTools:
    async def test_has_16_tools(self, client):
        tools = mcp._tool_manager._tools
        assert len(tools) == 16

    async def test_expected_tools_present(self, client):
        tool_names = set(mcp._tool_manager._tools.keys())
        for name in ["read_note", "create_note", "search", "server_info"]:
            assert name in tool_names


# --- CRUD ---


class TestCreateAndReadNote:
    async def test_create_then_read(self, client):
        result = await _call_tool(
            "create_note", path="tests/e2e-test.md", content="# E2E Test\nHello"
        )
        assert "Created" in result

        content = await _call_tool("read_note", path="tests/e2e-test.md")
        assert "# E2E Test" in content
        assert "Hello" in content


class TestAppendNote:
    async def test_append(self, client):
        await _call_tool("create_note", path="tests/e2e-append.md", content="Line 1")
        await _call_tool("append_note", path="tests/e2e-append.md", content="\nLine 2")

        content = await _call_tool("read_note", path="tests/e2e-append.md")
        assert "Line 1" in content
        assert "Line 2" in content


class TestDeleteNote:
    async def test_delete(self, client):
        await _call_tool("create_note", path="tests/e2e-delete.md", content="temp")
        result = await _call_tool("delete_note", path="tests/e2e-delete.md")
        assert "Deleted" in result

        with pytest.raises(Exception):
            await _call_tool("read_note", path="tests/e2e-delete.md")


class TestPatchNote:
    async def test_patch_heading(self, client):
        await _call_tool(
            "create_note",
            path="tests/e2e-patch.md",
            content="# Heading 1\nOriginal\n# Heading 2\nOther",
        )
        await _call_tool(
            "patch_note",
            path="tests/e2e-patch.md",
            operation="append",
            target_type="heading",
            target="Heading 1",
            content="\nPatched content",
        )
        content = await _call_tool("read_note", path="tests/e2e-patch.md")
        assert "Patched content" in content


# --- List files ---


class TestListFiles:
    async def test_list_test_folder(self, client):
        await _call_tool("create_note", path="tests/e2e-list.md", content="test")
        result = await _call_tool("list_files", path="tests")
        data = json.loads(result)
        assert "files" in data
        assert any("e2e-list" in f for f in data["files"])


# --- Search ---


class TestSearch:
    async def test_search(self, client):
        result = await _call_tool("search", query="test")
        data = json.loads(result)
        assert isinstance(data, list)


class TestSearchQuery:
    async def test_table_query(self, client):
        result = await _call_tool("search_query", query='TABLE file.name FROM ""')
        data = json.loads(result)
        assert isinstance(data, (list, dict))


# --- Server info ---


class TestServerInfo:
    async def test_status_ok(self, client):
        result = await _call_tool("server_info")
        data = json.loads(result)
        assert data["status"] == "OK"
        assert "versions" in data


# --- Commands ---


class TestListCommands:
    async def test_commands_present(self, client):
        result = await _call_tool("list_commands")
        data = json.loads(result)
        assert "commands" in data


class TestExecuteCommand:
    async def test_execute(self, client):
        result = await _call_tool("execute_command", command_id="app:go-back")
        assert "Executed command" in result


# --- Open file ---


class TestOpenFile:
    async def test_open(self, client):
        await _call_tool("create_note", path="tests/e2e-open.md", content="open me")
        result = await _call_tool("open_file", path="tests/e2e-open.md")
        assert "Opened" in result


# --- Periodic notes ---


class TestPeriodicNoteCrud:
    async def test_read_update_append(self, client):
        # Save original
        try:
            original = await _call_tool("get_periodic_note", period="daily")
        except Exception:
            pytest.skip("Daily note not configured")

        try:
            # Update
            result = await _call_tool(
                "update_periodic_note", period="daily", content="E2E test content"
            )
            assert "Updated" in result

            # Read back
            content = await _call_tool("get_periodic_note", period="daily")
            assert "E2E test content" in content

            # Append
            result = await _call_tool("append_periodic_note", period="daily", content="\nAppended")
            assert "Appended" in result
        finally:
            # Restore original
            await _call_tool("update_periodic_note", period="daily", content=original)


class TestPatchPeriodicNote:
    async def test_patch_heading(self, client):
        try:
            original = await _call_tool("get_periodic_note", period="daily")
        except Exception:
            pytest.skip("Daily note not configured")

        # Check for an actual heading, not just the word "Tasks" in body text
        has_tasks_heading = any(
            line.lstrip().startswith("#") and "Tasks" in line for line in original.splitlines()
        )
        if not has_tasks_heading:
            pytest.skip("Daily note has no Tasks heading")

        try:
            try:
                await _call_tool(
                    "patch_periodic_note",
                    period="daily",
                    operation="append",
                    target_type="heading",
                    target="Tasks",
                    content="\n- [ ] E2E test task",
                )
            except Exception as e:
                if "invalid-target" in str(e):
                    pytest.skip("Tasks heading not patchable (template-generated or nested)")
                raise
            content = await _call_tool("get_periodic_note", period="daily")
            assert "E2E test task" in content
        finally:
            await _call_tool("update_periodic_note", period="daily", content=original)


# --- Error propagation ---


class TestErrorPropagation:
    async def test_read_nonexistent(self, client):
        with pytest.raises(Exception) as exc_info:
            await _call_tool("read_note", path="nonexistent/path/does-not-exist.md")
        assert "404" in str(exc_info.value)
