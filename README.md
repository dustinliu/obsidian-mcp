# obsidian-mcp

An MCP (Model Context Protocol) server that exposes Obsidian vault operations as tools for AI assistants. Supports both stdio (default) and Streamable HTTP transport modes. Communicates with Obsidian through the [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) plugin.

## Prerequisites

- [Obsidian](https://obsidian.md/) with [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) plugin installed and enabled
- Rust toolchain (for building from source)

## Build

```bash
cargo build --release
```

## Usage

```bash
obsidian-mcp --api-key <YOUR_API_KEY>
```

### Options

| Flag | Env Var | Default | Description |
|------|---------|---------|-------------|
| `--api-url` | `OBSIDIAN_API_URL` | `https://127.0.0.1:27124` | Obsidian REST API URL |
| `--api-key` | `OBSIDIAN_API_KEY` | (required) | Obsidian REST API key |
| `--transport` | `MCP_TRANSPORT` | `stdio` | Transport mode: `stdio` or `http` |
| `--port` | `MCP_PORT` | `3000` | MCP server listen port |
| `--host` | `MCP_HOST` | `127.0.0.1` | MCP server listen host |

### MCP Client Configuration

**stdio (default):** Configure your MCP client to spawn the server directly:

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

**HTTP:** Connect your MCP client to `http://127.0.0.1:3000/mcp`:

```bash
obsidian-mcp --api-key <YOUR_API_KEY> --transport http
```

## Tools

| Tool | Description |
|------|-------------|
| `read_note` | Read the content of a note |
| `create_note` | Create a new note or overwrite an existing one |
| `append_note` | Append content to an existing note |
| `patch_note` | Partially update a note relative to a heading, block reference, or frontmatter field. For `append` operations, a trailing newline is added automatically. Pass `content_type: "application/json"` to set frontmatter fields to structured values (e.g. arrays). |
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
| `patch_periodic_note` | Partially update a periodic note relative to a heading, block reference, or frontmatter field. Same `append` newline and `content_type` behavior as `patch_note`. |
| `server_info` | Get Obsidian API server status |

## License

MIT
