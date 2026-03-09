"""Tests for CLI entrypoint and transport selection."""

from unittest.mock import AsyncMock, patch

import pytest
from click.testing import CliRunner

from obsidian_mcp.__main__ import main, _run
from obsidian_mcp.server import mcp


# --- CLI option tests (mock _run to isolate click parsing) ---


@patch("obsidian_mcp.__main__._run", new_callable=AsyncMock)
class TestTransportCLI:
    """Test --transport CLI option parsing."""

    def _invoke(
        self, mock_run: AsyncMock, args: list[str] | None = None, env: dict[str, str] | None = None
    ) -> str:
        """Invoke CLI and return the transport value passed to _run."""
        runner = CliRunner(env=env) if env else CliRunner()
        result = runner.invoke(main, ["--api-key", "test-key", *(args or [])])
        assert result.exit_code == 0
        _, kwargs = mock_run.call_args
        return kwargs["transport"]

    def test_default_is_stdio(self, mock_run: AsyncMock) -> None:
        assert self._invoke(mock_run) == "stdio"

    def test_explicit_stdio(self, mock_run: AsyncMock) -> None:
        assert self._invoke(mock_run, ["--transport", "stdio"]) == "stdio"

    def test_explicit_http(self, mock_run: AsyncMock) -> None:
        assert self._invoke(mock_run, ["--transport", "http"]) == "http"

    def test_invalid_transport(self, mock_run: AsyncMock) -> None:
        runner = CliRunner()
        result = runner.invoke(main, ["--api-key", "test-key", "--transport", "invalid"])
        assert result.exit_code != 0
        mock_run.assert_not_called()

    def test_env_var_override(self, mock_run: AsyncMock) -> None:
        assert self._invoke(mock_run, env={"MCP_TRANSPORT": "http"}) == "http"


# --- Transport branching tests (mock mcp methods to verify _run logic) ---


class TestTransportBranching:
    """Test that _run() calls the correct transport method."""

    @pytest.fixture()
    def mock_client(self):
        client = AsyncMock()
        client.server_info.return_value = AsyncMock(status="ok")
        return client

    @pytest.mark.asyncio
    @patch.object(mcp, "run_stdio_async", new_callable=AsyncMock)
    async def test_stdio_calls_run_stdio(
        self, mock_stdio: AsyncMock, mock_client: AsyncMock
    ) -> None:
        with (
            patch("obsidian_mcp.__main__.ObsidianClient") as MockClient,
            patch("obsidian_mcp.__main__.set_client"),
        ):
            MockClient.return_value.__aenter__ = AsyncMock(return_value=mock_client)
            MockClient.return_value.__aexit__ = AsyncMock(return_value=False)
            await _run(
                api_url="https://localhost",
                api_key="k",
                port=3000,
                host="127.0.0.1",
                transport="stdio",
            )
        mock_stdio.assert_called_once()

    @pytest.mark.asyncio
    @patch.object(mcp, "run_streamable_http_async", new_callable=AsyncMock)
    async def test_http_calls_run_streamable_http(
        self, mock_http: AsyncMock, mock_client: AsyncMock
    ) -> None:
        with (
            patch("obsidian_mcp.__main__.ObsidianClient") as MockClient,
            patch("obsidian_mcp.__main__.set_client"),
        ):
            MockClient.return_value.__aenter__ = AsyncMock(return_value=mock_client)
            MockClient.return_value.__aexit__ = AsyncMock(return_value=False)
            await _run(
                api_url="https://localhost",
                api_key="k",
                port=3000,
                host="127.0.0.1",
                transport="http",
            )
        mock_http.assert_called_once()
        assert mcp.settings.host == "127.0.0.1"
        assert mcp.settings.port == 3000
