## MODIFIED Requirements

### Requirement: Tool definitions
The MCP server SHALL expose exactly 16 tools using the `mcp` Python SDK's `@server.tool()` decorator. Each tool SHALL accept arguments defined as Pydantic `BaseModel` classes with `Field()` descriptions for JSON Schema generation.

#### Scenario: All 16 tools registered
- **WHEN** the MCP server starts
- **THEN** exactly 16 tools SHALL be available with the same names and descriptions as the Rust implementation

### Requirement: Tool result format
Tool results SHALL return text content. Success responses SHALL be plain strings (confirmation messages, markdown content) or JSON-serialized strings (for list/search/info tools). Errors from `ObsidianClient` SHALL be converted to MCP errors.

#### Scenario: Successful tool call
- **WHEN** a tool is called and the underlying client method succeeds
- **THEN** the result SHALL contain a text content block with the appropriate value

#### Scenario: Client error propagation
- **WHEN** a tool is called and the underlying client method raises `AppError`
- **THEN** the tool SHALL raise an MCP error with the error message

### Requirement: search_query validation
The `search_query` tool SHALL validate that the query starts with `"TABLE"` (after trimming leading whitespace) before calling `ObsidianClient`. Non-TABLE queries SHALL return an invalid params error immediately.

#### Scenario: Non-TABLE query rejected
- **WHEN** `search_query` is called with a query not starting with "TABLE"
- **THEN** an invalid params MCP error SHALL be returned without calling the client

### Requirement: Server info
The server SHALL report name `"obsidian-mcp"`, version from package metadata, and capabilities for tools only (no resources, no prompts).

#### Scenario: Server info returned
- **WHEN** the MCP client requests server info
- **THEN** the server name SHALL be `"obsidian-mcp"` and tools capability SHALL be enabled
