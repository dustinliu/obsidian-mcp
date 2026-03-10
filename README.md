# obsidian-mcp

An MCP (Model Context Protocol) server that lets AI assistants read and write your Obsidian vault. Connect Claude Desktop or any MCP-compatible client to your Obsidian notes.

## Prerequisites

### 1. Obsidian with Local REST API plugin

1. Open Obsidian → **Settings** → **Community plugins** → Browse
2. Search for **Local REST API** and install it
3. Enable the plugin and open its settings
4. Copy the **API Key** — you will need it in the next step

### 2. Rust toolchain (to build from source)

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Installation

```bash
cargo install --path .
```

Or build and copy to `~/.local/bin` with [just](https://github.com/casey/just):

```bash
just deploy
```

## Configuration

### stdio (recommended for Claude Desktop)

Add the following to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "obsidian": {
      "command": "obsidian-mcp",
      "args": ["--api-key", "<YOUR_API_KEY>"]
    }
  }
}
```

Or use the environment variable instead of the flag:

```json
{
  "mcpServers": {
    "obsidian": {
      "command": "obsidian-mcp",
      "env": {
        "OBSIDIAN_API_KEY": "<YOUR_API_KEY>"
      }
    }
  }
}
```

### HTTP transport (for remote or multi-client setups)

Start the server in HTTP mode:

```bash
obsidian-mcp --api-key <YOUR_API_KEY> --transport http
```

Then point your MCP client at `http://127.0.0.1:3000/mcp`.

## Options

| Flag | Env Var | Default | Description |
|------|---------|---------|-------------|
| `--api-url` | `OBSIDIAN_API_URL` | `https://127.0.0.1:27124` | Obsidian REST API URL |
| `--api-key` | `OBSIDIAN_API_KEY` | *(required)* | Obsidian REST API key |
| `--transport` | `MCP_TRANSPORT` | `stdio` | Transport mode: `stdio` or `http` |
| `--port` | `MCP_PORT` | `3000` | HTTP server listen port |
| `--host` | `MCP_HOST` | `127.0.0.1` | HTTP server listen host |

## Available Tools

Once connected, the AI assistant can use the following tools to interact with your vault:

### Notes

| Tool | Description |
|------|-------------|
| `read_note` | Read the content of a note |
| `create_note` | Create a new note or overwrite an existing one |
| `append_note` | Append content to the end of a note |
| `patch_note` | Partially update a note at a specific heading, block reference, or frontmatter field. For `append` operations a trailing newline is added automatically. Pass `content_type: "application/json"` to set frontmatter fields to structured values (e.g. arrays). |
| `delete_note` | Delete a note from the vault |
| `list_files` | List files in a vault directory |

### Search

| Tool | Description |
|------|-------------|
| `search` | Search notes by text query |
| `search_query` | Search notes using a Dataview DQL query. Only `TABLE` queries are supported; `LIST` and `TASK` query types are not supported by the Obsidian Local REST API. |

### Periodic Notes

| Tool | Description |
|------|-------------|
| `get_periodic_note` | Read a periodic note (daily, weekly, monthly, quarterly, or yearly) |
| `update_periodic_note` | Replace the entire content of a periodic note |
| `append_periodic_note` | Append content to a periodic note |
| `patch_periodic_note` | Partially update a periodic note at a heading, block reference, or frontmatter field. Same `append` and `content_type` behavior as `patch_note`. |

### Obsidian UI

| Tool | Description |
|------|-------------|
| `list_commands` | List all available Obsidian commands |
| `execute_command` | Execute an Obsidian command by its ID |
| `open_file` | Open a file in the Obsidian UI |
| `server_info` | Get Obsidian Local REST API server status and version |

## License

MIT
