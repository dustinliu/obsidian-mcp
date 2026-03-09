"""Tests for ObsidianClient using respx to mock httpx."""

import httpx
import pytest
import respx

from obsidian_mcp.client import ObsidianClient
from obsidian_mcp.errors import ApiError, HttpError
from obsidian_mcp.types import Operation, PatchParams, TargetType

BASE_URL = "https://127.0.0.1:27124"
API_KEY = "test-key"


@pytest.fixture
def client():
    return ObsidianClient(BASE_URL, API_KEY)


# --- Construction ---


class TestConstruction:
    def test_bearer_token_formatted(self, client):
        assert client._bearer_token == "Bearer test-key"

    def test_base_url_stored(self, client):
        assert client._base_url == BASE_URL


# --- URL helpers ---


class TestPeriodicUrl:
    def test_with_all_date_params(self, client):
        url = client._periodic_url("daily", 2026, 3, 6)
        assert url == "/periodic/daily/2026/3/6/"

    def test_without_date_params(self, client):
        url = client._periodic_url("daily", None, None, None)
        assert url == "/periodic/daily/"

    def test_partial_date_falls_back_to_period_only(self, client):
        assert client._periodic_url("daily", 2026, None, None) == "/periodic/daily/"
        assert client._periodic_url("daily", 2026, 3, None) == "/periodic/daily/"
        assert client._periodic_url("daily", None, 3, 6) == "/periodic/daily/"


# --- Async context manager ---


class TestAsyncContextManager:
    async def test_aenter_returns_self(self):
        client = ObsidianClient(BASE_URL, API_KEY)
        async with client as c:
            assert c is client


# --- Server info ---


class TestServerInfo:
    @respx.mock
    async def test_success(self, client):
        respx.get(f"{BASE_URL}/").mock(
            return_value=httpx.Response(200, json={"status": "OK", "versions": {"api": "1.0"}})
        )
        info = await client.server_info()
        assert info.status == "OK"
        assert info.versions == {"api": "1.0"}


# --- Vault notes ---


class TestReadNote:
    @respx.mock
    async def test_success(self, client):
        respx.get(f"{BASE_URL}/vault/folder/note.md").mock(
            return_value=httpx.Response(200, text="# Hello\nContent")
        )
        result = await client.read_note("folder/note.md")
        assert result == "# Hello\nContent"

    @respx.mock
    async def test_sends_accept_markdown(self, client):
        route = respx.get(f"{BASE_URL}/vault/note.md").mock(
            return_value=httpx.Response(200, text="content")
        )
        await client.read_note("note.md")
        assert route.calls[0].request.headers["accept"] == "text/markdown"


class TestCreateNote:
    @respx.mock
    async def test_sends_put_with_markdown(self, client):
        route = respx.put(f"{BASE_URL}/vault/new.md").mock(return_value=httpx.Response(200))
        await client.create_note("new.md", "# New Note")
        req = route.calls[0].request
        assert req.headers["content-type"] == "text/markdown"
        assert req.content == b"# New Note"


class TestAppendNote:
    @respx.mock
    async def test_sends_post_with_markdown(self, client):
        route = respx.post(f"{BASE_URL}/vault/existing.md").mock(return_value=httpx.Response(200))
        await client.append_note("existing.md", "More content")
        req = route.calls[0].request
        assert req.headers["content-type"] == "text/markdown"
        assert req.content == b"More content"


class TestPatchNote:
    @respx.mock
    async def test_sends_v3_headers(self, client):
        route = respx.patch(f"{BASE_URL}/vault/note.md").mock(
            return_value=httpx.Response(200, text="patched content")
        )
        params = PatchParams(
            operation=Operation.APPEND,
            target_type=TargetType.HEADING,
            target="Section 1",
        )
        result = await client.patch_note("note.md", params, "new text")
        req = route.calls[0].request
        assert req.headers["operation"] == "append"
        assert req.headers["target-type"] == "heading"
        assert req.headers["target"] == "Section 1"
        assert result == "patched content"

    @respx.mock
    async def test_sends_optional_headers(self, client):
        route = respx.patch(f"{BASE_URL}/vault/note.md").mock(
            return_value=httpx.Response(200, text="ok")
        )
        params = PatchParams(
            operation=Operation.REPLACE,
            target_type=TargetType.FRONTMATTER,
            target="tags",
            target_delimiter="::",
            trim_target_whitespace=True,
            create_target_if_missing=True,
        )
        await client.patch_note("note.md", params, "content")
        req = route.calls[0].request
        assert req.headers["target-delimiter"] == "::"
        assert req.headers["trim-target-whitespace"] == "true"
        assert req.headers["create-target-if-missing"] == "true"


class TestDeleteNote:
    @respx.mock
    async def test_sends_delete(self, client):
        respx.delete(f"{BASE_URL}/vault/old.md").mock(return_value=httpx.Response(200))
        await client.delete_note("old.md")


class TestListFiles:
    @respx.mock
    async def test_root(self, client):
        respx.get(f"{BASE_URL}/vault/").mock(
            return_value=httpx.Response(200, json={"files": ["a.md", "b.md"]})
        )
        result = await client.list_files()
        assert result == {"files": ["a.md", "b.md"]}

    @respx.mock
    async def test_subdirectory(self, client):
        route = respx.get(f"{BASE_URL}/vault/subdir/").mock(
            return_value=httpx.Response(200, json={"files": ["c.md"]})
        )
        result = await client.list_files("subdir")
        assert result == {"files": ["c.md"]}
        assert route.calls[0].request.headers["accept"] == "application/json"


# --- Search ---


class TestSearchSimple:
    @respx.mock
    async def test_success(self, client):
        respx.post(f"{BASE_URL}/search/simple/").mock(
            return_value=httpx.Response(200, json=[{"filename": "note.md"}])
        )
        result = await client.search_simple("test query")
        assert result == [{"filename": "note.md"}]

    @respx.mock
    async def test_sends_query_param(self, client):
        route = respx.post(f"{BASE_URL}/search/simple/").mock(
            return_value=httpx.Response(200, json=[])
        )
        await client.search_simple("my search")
        assert route.calls[0].request.url.params["query"] == "my search"


class TestSearchQuery:
    @respx.mock
    async def test_success(self, client):
        respx.post(f"{BASE_URL}/search/").mock(
            return_value=httpx.Response(200, json=[{"file": {"name": "note.md"}}])
        )
        result = await client.search_query('TABLE file.name FROM ""')
        assert isinstance(result, list)

    @respx.mock
    async def test_sends_dql_content_type(self, client):
        route = respx.post(f"{BASE_URL}/search/").mock(return_value=httpx.Response(200, json=[]))
        await client.search_query("TABLE file.ctime")
        req = route.calls[0].request
        assert req.headers["content-type"] == "application/vnd.olrapi.dataview.dql+txt"
        assert req.content == b"TABLE file.ctime"


# --- Commands ---


class TestListCommands:
    @respx.mock
    async def test_success(self, client):
        respx.get(f"{BASE_URL}/commands/").mock(
            return_value=httpx.Response(200, json={"commands": [{"id": "cmd1"}]})
        )
        result = await client.list_commands()
        assert result == {"commands": [{"id": "cmd1"}]}


class TestExecuteCommand:
    @respx.mock
    async def test_sends_post(self, client):
        route = respx.post(f"{BASE_URL}/commands/editor:toggle-bold/").mock(
            return_value=httpx.Response(200)
        )
        await client.execute_command("editor:toggle-bold")
        assert route.called


# --- UI ---


class TestOpenFile:
    @respx.mock
    async def test_sends_post(self, client):
        route = respx.post(f"{BASE_URL}/open/my-note.md").mock(return_value=httpx.Response(200))
        await client.open_file("my-note.md")
        assert route.called


# --- Periodic notes ---


class TestGetPeriodicNote:
    @respx.mock
    async def test_without_date(self, client):
        respx.get(f"{BASE_URL}/periodic/daily/").mock(
            return_value=httpx.Response(200, text="Daily note content")
        )
        result = await client.get_periodic_note("daily")
        assert result == "Daily note content"

    @respx.mock
    async def test_with_date(self, client):
        respx.get(f"{BASE_URL}/periodic/daily/2026/3/6/").mock(
            return_value=httpx.Response(200, text="Dated content")
        )
        result = await client.get_periodic_note("daily", 2026, 3, 6)
        assert result == "Dated content"


class TestUpdatePeriodicNote:
    @respx.mock
    async def test_sends_put(self, client):
        route = respx.put(f"{BASE_URL}/periodic/weekly/2026/3/6/").mock(
            return_value=httpx.Response(200)
        )
        await client.update_periodic_note("weekly", 2026, 3, 6, "new content")
        req = route.calls[0].request
        assert req.headers["content-type"] == "text/markdown"
        assert req.content == b"new content"


class TestAppendPeriodicNote:
    @respx.mock
    async def test_sends_post(self, client):
        route = respx.post(f"{BASE_URL}/periodic/daily/2026/3/6/").mock(
            return_value=httpx.Response(200)
        )
        await client.append_periodic_note("daily", 2026, 3, 6, "appended")
        assert route.calls[0].request.content == b"appended"


class TestPatchPeriodicNote:
    @respx.mock
    async def test_sends_v3_headers(self, client):
        route = respx.patch(f"{BASE_URL}/periodic/daily/2026/3/6/").mock(
            return_value=httpx.Response(200, text="patched")
        )
        params = PatchParams(
            operation=Operation.APPEND,
            target_type=TargetType.HEADING,
            target="Tasks",
        )
        result = await client.patch_periodic_note(
            "daily", 2026, 3, 6, params=params, content="- [ ] task"
        )
        assert result == "patched"
        req = route.calls[0].request
        assert req.headers["operation"] == "append"
        assert req.headers["target-type"] == "heading"
        assert req.headers["target"] == "Tasks"

    @respx.mock
    async def test_without_date(self, client):
        respx.patch(f"{BASE_URL}/periodic/monthly/").mock(
            return_value=httpx.Response(200, text="ok")
        )
        params = PatchParams(
            operation=Operation.REPLACE,
            target_type=TargetType.FRONTMATTER,
            target="status",
        )
        result = await client.patch_periodic_note("monthly", params=params, content="done")
        assert result == "ok"


# --- Error handling ---


class TestErrorHandling:
    @respx.mock
    async def test_api_error_on_non_success_status(self, client):
        respx.get(f"{BASE_URL}/vault/missing.md").mock(
            return_value=httpx.Response(404, text="File not found")
        )
        with pytest.raises(ApiError) as exc_info:
            await client.read_note("missing.md")
        assert exc_info.value.status == 404
        assert exc_info.value.body == "File not found"

    @respx.mock
    async def test_http_error_on_connection_failure(self, client):
        respx.get(f"{BASE_URL}/").mock(side_effect=httpx.ConnectError("refused"))
        with pytest.raises(HttpError):
            await client.server_info()
