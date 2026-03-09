"""HTTP client for the Obsidian Local REST API."""

from dataclasses import dataclass
from typing import Any

import httpx

from obsidian_mcp.errors import ApiError, HttpError, JsonError
from obsidian_mcp.types import PatchParams


@dataclass
class ServerInfo:
    """Obsidian REST API server status."""

    status: str
    versions: dict[str, Any]


class ObsidianClient:
    """Wraps httpx.AsyncClient to call the Obsidian Local REST API."""

    def __init__(self, base_url: str, api_key: str) -> None:
        self._base_url = base_url.rstrip("/")
        self._bearer_token = f"Bearer {api_key}"
        self._client = httpx.AsyncClient(
            base_url=self._base_url,
            verify=False,
            headers={"Authorization": self._bearer_token},
        )

    async def __aenter__(self) -> "ObsidianClient":
        return self

    async def __aexit__(self, *args: object) -> None:
        await self._client.aclose()

    async def _check_response(self, resp: httpx.Response) -> httpx.Response:
        if resp.is_success:
            return resp
        body = resp.text
        raise ApiError(resp.status_code, body)

    def _periodic_url(
        self,
        period: str,
        year: int | None,
        month: int | None,
        day: int | None,
    ) -> str:
        if year is not None and month is not None and day is not None:
            return f"/periodic/{period}/{year}/{month}/{day}/"
        return f"/periodic/{period}/"

    def _patch_headers(self, params: PatchParams) -> dict[str, str]:
        headers: dict[str, str] = {
            "Content-Type": "text/markdown",
            "Operation": str(params.operation),
            "Target-Type": str(params.target_type),
            "Target": params.target,
        }
        if params.target_delimiter is not None:
            headers["Target-Delimiter"] = params.target_delimiter
        if params.trim_target_whitespace is not None:
            headers["Trim-Target-Whitespace"] = str(params.trim_target_whitespace).lower()
        if params.create_target_if_missing is not None:
            headers["Create-Target-If-Missing"] = str(params.create_target_if_missing).lower()
        return headers

    # --- Health ---

    async def server_info(self) -> ServerInfo:
        try:
            resp = await self._client.get("/")
            resp = await self._check_response(resp)
            data = resp.json()
            return ServerInfo(
                status=data.get("status", ""),
                versions=data.get("versions", {}),
            )
        except httpx.HTTPError as e:
            raise HttpError(e) from e
        except ValueError as e:
            raise JsonError(e) from e

    # --- Vault notes ---

    async def read_note(self, path: str) -> str:
        try:
            resp = await self._client.get(
                f"/vault/{path}",
                headers={"Accept": "text/markdown"},
            )
            resp = await self._check_response(resp)
            return resp.text
        except httpx.HTTPError as e:
            raise HttpError(e) from e

    async def create_note(self, path: str, content: str) -> None:
        try:
            resp = await self._client.put(
                f"/vault/{path}",
                content=content,
                headers={"Content-Type": "text/markdown"},
            )
            await self._check_response(resp)
        except httpx.HTTPError as e:
            raise HttpError(e) from e

    async def append_note(self, path: str, content: str) -> None:
        try:
            resp = await self._client.post(
                f"/vault/{path}",
                content=content,
                headers={"Content-Type": "text/markdown"},
            )
            await self._check_response(resp)
        except httpx.HTTPError as e:
            raise HttpError(e) from e

    async def patch_note(self, path: str, params: PatchParams, content: str) -> str:
        try:
            resp = await self._client.patch(
                f"/vault/{path}",
                content=content,
                headers=self._patch_headers(params),
            )
            resp = await self._check_response(resp)
            return resp.text
        except httpx.HTTPError as e:
            raise HttpError(e) from e

    async def delete_note(self, path: str) -> None:
        try:
            resp = await self._client.delete(f"/vault/{path}")
            await self._check_response(resp)
        except httpx.HTTPError as e:
            raise HttpError(e) from e

    async def list_files(self, path: str | None = None) -> Any:
        try:
            url = f"/vault/{path}/" if path else "/vault/"
            resp = await self._client.get(
                url,
                headers={"Accept": "application/json"},
            )
            resp = await self._check_response(resp)
            return resp.json()
        except httpx.HTTPError as e:
            raise HttpError(e) from e
        except ValueError as e:
            raise JsonError(e) from e

    # --- Search ---

    async def search_simple(self, query: str) -> Any:
        try:
            resp = await self._client.post(
                "/search/simple/",
                params={"query": query},
            )
            resp = await self._check_response(resp)
            return resp.json()
        except httpx.HTTPError as e:
            raise HttpError(e) from e
        except ValueError as e:
            raise JsonError(e) from e

    async def search_query(self, query: str) -> Any:
        try:
            resp = await self._client.post(
                "/search/",
                content=query,
                headers={"Content-Type": "application/vnd.olrapi.dataview.dql+txt"},
            )
            resp = await self._check_response(resp)
            return resp.json()
        except httpx.HTTPError as e:
            raise HttpError(e) from e
        except ValueError as e:
            raise JsonError(e) from e

    # --- Commands ---

    async def list_commands(self) -> Any:
        try:
            resp = await self._client.get("/commands/")
            resp = await self._check_response(resp)
            return resp.json()
        except httpx.HTTPError as e:
            raise HttpError(e) from e
        except ValueError as e:
            raise JsonError(e) from e

    async def execute_command(self, command_id: str) -> None:
        try:
            resp = await self._client.post(f"/commands/{command_id}/")
            await self._check_response(resp)
        except httpx.HTTPError as e:
            raise HttpError(e) from e

    # --- UI ---

    async def open_file(self, filename: str) -> None:
        try:
            resp = await self._client.post(f"/open/{filename}")
            await self._check_response(resp)
        except httpx.HTTPError as e:
            raise HttpError(e) from e

    # --- Periodic notes ---

    async def get_periodic_note(
        self,
        period: str,
        year: int | None = None,
        month: int | None = None,
        day: int | None = None,
    ) -> str:
        try:
            url = self._periodic_url(period, year, month, day)
            resp = await self._client.get(
                url,
                headers={"Accept": "text/markdown"},
            )
            resp = await self._check_response(resp)
            return resp.text
        except httpx.HTTPError as e:
            raise HttpError(e) from e

    async def update_periodic_note(
        self,
        period: str,
        year: int | None = None,
        month: int | None = None,
        day: int | None = None,
        content: str = "",
    ) -> None:
        try:
            url = self._periodic_url(period, year, month, day)
            resp = await self._client.put(
                url,
                content=content,
                headers={"Content-Type": "text/markdown"},
            )
            await self._check_response(resp)
        except httpx.HTTPError as e:
            raise HttpError(e) from e

    async def append_periodic_note(
        self,
        period: str,
        year: int | None = None,
        month: int | None = None,
        day: int | None = None,
        content: str = "",
    ) -> None:
        try:
            url = self._periodic_url(period, year, month, day)
            resp = await self._client.post(
                url,
                content=content,
                headers={"Content-Type": "text/markdown"},
            )
            await self._check_response(resp)
        except httpx.HTTPError as e:
            raise HttpError(e) from e

    async def patch_periodic_note(
        self,
        period: str,
        year: int | None = None,
        month: int | None = None,
        day: int | None = None,
        params: PatchParams | None = None,
        content: str = "",
    ) -> str:
        if params is None:
            raise ValueError("params is required")
        try:
            url = self._periodic_url(period, year, month, day)
            resp = await self._client.patch(
                url,
                content=content,
                headers=self._patch_headers(params),
            )
            resp = await self._check_response(resp)
            return resp.text
        except httpx.HTTPError as e:
            raise HttpError(e) from e
