# obsidian-mcp

An MCP (Model Context Protocol) server that exposes Obsidian vault operations as tools for AI assistants. Communicates with Obsidian through the [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) plugin.

## Prerequisites

- [Obsidian](https://obsidian.md/) with [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) plugin installed and enabled
- Python ≥ 3.12
- [uv](https://docs.astral.sh/uv/) (recommended) or pip

## Install

```bash
uv tool install .
```

## Usage

```bash
obsidian-mcp --api-key <YOUR_API_KEY>
# or
uv run obsidian-mcp --api-key <YOUR_API_KEY>
```

### Options

| Flag | Env Var | Default | Description |
|------|---------|---------|-------------|
| `--api-url` | `OBSIDIAN_API_URL` | `https://127.0.0.1:27124` | Obsidian REST API URL |
| `--api-key` | `OBSIDIAN_API_KEY` | (required) | Obsidian REST API key |
| `--port` | `MCP_PORT` | `3000` | MCP server listen port |
| `--host` | `MCP_HOST` | `127.0.0.1` | MCP server listen host |

### MCP Client Configuration

Connect your MCP client to `http://127.0.0.1:3000/mcp`.

## Tools

| Tool | Description |
|------|-------------|
| `read_note` | Read the content of a note |
| `create_note` | Create a new note or overwrite an existing one |
| `append_note` | Append content to an existing note |
| `patch_note` | Partially update a note relative to a heading, block reference, or frontmatter field |
| `delete_note` | Delete a note from the vault |
| `list_files` | List files in a vault directory |
| `search` | Search notes by text query |
| `search_query` | Search notes using Dataview DQL query |
| `list_commands` | List all available Obsidian commands |
| `execute_command` | Execute an Obsidian command by ID |
| `open_file` | Open a file in the Obsidian UI |
| `get_periodic_note` | Read a periodic note |
| `update_periodic_note` | Replace the content of a periodic note |
| `append_periodic_note` | Append content to a periodic note |
| `patch_periodic_note` | Partially update a periodic note relative to a heading, block reference, or frontmatter field |
| `server_info` | Get Obsidian API server status |

## License

MIT
