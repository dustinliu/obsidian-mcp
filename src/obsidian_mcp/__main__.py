"""CLI entrypoint for obsidian-mcp server."""

import asyncio
import logging
import sys

import click

from obsidian_mcp.client import ObsidianClient
from obsidian_mcp.server import mcp, set_client

logger = logging.getLogger("obsidian_mcp")


@click.command()
@click.option(
    "--api-url",
    envvar="OBSIDIAN_API_URL",
    default="https://127.0.0.1:27124",
    help="Obsidian Local REST API URL",
)
@click.option(
    "--api-key",
    envvar="OBSIDIAN_API_KEY",
    required=True,
    help="Obsidian Local REST API key",
)
@click.option(
    "--port",
    envvar="MCP_PORT",
    default=3000,
    type=int,
    help="MCP server port",
)
@click.option(
    "--host",
    envvar="MCP_HOST",
    default="127.0.0.1",
    help="MCP server host",
)
def main(api_url: str, api_key: str, port: int, host: str) -> None:
    """MCP server for Obsidian vault operations."""
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
    )
    asyncio.run(_run(api_url, api_key, port, host))


async def _run(api_url: str, api_key: str, port: int, host: str) -> None:
    async with ObsidianClient(api_url, api_key) as client:
        # Verify connectivity at startup
        try:
            info = await client.server_info()
            logger.info("Connected to Obsidian: status=%s", info.status)
        except Exception as e:
            logger.error("Failed to connect to Obsidian at %s: %s", api_url, e)
            sys.exit(1)

        set_client(client)

        mcp.settings.host = host
        mcp.settings.port = port

        logger.info("Starting MCP server on %s:%d", host, port)
        await mcp.run_streamable_http_async()


if __name__ == "__main__":
    main()
