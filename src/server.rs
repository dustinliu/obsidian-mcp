use std::sync::Arc;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::ObsidianClient;
use crate::types::{Operation, PatchParams, TargetType};

fn to_mcp_error(e: impl std::fmt::Display) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

#[derive(Clone)]
pub struct ObsidianServer {
    client: Arc<ObsidianClient>,
    tool_router: ToolRouter<Self>,
}

// --- Arg structs ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadNoteArgs {
    /// Path to the note, e.g. "folder/note.md"
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateNoteArgs {
    /// Path for the new note, e.g. "folder/note.md"
    pub path: String,
    /// Markdown content for the note
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AppendNoteArgs {
    /// Path to the note to append to
    pub path: String,
    /// Content to append
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PatchNoteArgs {
    /// Path to the note to patch
    pub path: String,
    /// Patch operation: "append", "prepend", or "replace"
    pub operation: Operation,
    /// Target type: "heading", "block", or "frontmatter"
    pub target_type: TargetType,
    /// Target identifier. For headings: use the heading text without the # prefix.
    /// Sub-headings require the full path from the top-level heading using :: as delimiter
    /// (e.g. "Heading 1::Subheading 1" to target ## Subheading 1 under # Heading 1).
    /// For block references: the block ID. For frontmatter: the field name.
    pub target: String,
    /// Delimiter for nested heading paths (default: "::")
    pub target_delimiter: Option<String>,
    /// Trim whitespace from target before applying patch
    pub trim_target_whitespace: Option<bool>,
    /// Create the target if it doesn't exist (useful for frontmatter)
    pub create_target_if_missing: Option<bool>,
    /// Content-Type for the request body. Use "application/json" when setting frontmatter fields to structured values like arrays (e.g. ["tag1","tag2"]). Defaults to "text/markdown".
    pub content_type: Option<String>,
    /// Content to insert at the target location
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteNoteArgs {
    /// Path to the note to delete
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListFilesArgs {
    /// Directory path to list (omit for vault root)
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchArgs {
    /// Search query string
    pub query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExecuteCommandArgs {
    /// The command ID to execute
    pub command_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenFileArgs {
    /// Path to the file to open in Obsidian UI
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPeriodicNoteArgs {
    /// Period type: "daily", "weekly", "monthly", "quarterly", "yearly"
    pub period: String,
    /// Year (optional, omit for current period)
    pub year: Option<u32>,
    /// Month (optional)
    pub month: Option<u32>,
    /// Day (optional)
    pub day: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdatePeriodicNoteArgs {
    /// Period type: "daily", "weekly", "monthly", "quarterly", "yearly"
    pub period: String,
    /// Year (optional, omit for current period)
    pub year: Option<u32>,
    /// Month (optional)
    pub month: Option<u32>,
    /// Day (optional)
    pub day: Option<u32>,
    /// New content to replace the entire note
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AppendPeriodicNoteArgs {
    /// Period type: "daily", "weekly", "monthly", "quarterly", "yearly"
    pub period: String,
    /// Year (optional, omit for current period)
    pub year: Option<u32>,
    /// Month (optional)
    pub month: Option<u32>,
    /// Day (optional)
    pub day: Option<u32>,
    /// Content to append
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PatchPeriodicNoteArgs {
    /// Period type: "daily", "weekly", "monthly", "quarterly", "yearly"
    pub period: String,
    /// Year (optional, omit for current period)
    pub year: Option<u32>,
    /// Month (optional)
    pub month: Option<u32>,
    /// Day (optional)
    pub day: Option<u32>,
    /// Patch operation: "append", "prepend", or "replace"
    pub operation: Operation,
    /// Target type: "heading", "block", or "frontmatter"
    pub target_type: TargetType,
    /// Target identifier. For headings: use the heading text without the # prefix.
    /// Sub-headings require the full path from the top-level heading using :: as delimiter
    /// (e.g. "Heading 1::Subheading 1" to target ## Subheading 1 under # Heading 1).
    /// For block references: the block ID. For frontmatter: the field name.
    pub target: String,
    /// Delimiter for nested heading paths (default: "::")
    pub target_delimiter: Option<String>,
    /// Trim whitespace from target before applying patch
    pub trim_target_whitespace: Option<bool>,
    /// Create the target if it doesn't exist (useful for frontmatter)
    pub create_target_if_missing: Option<bool>,
    /// Content-Type for the request body. Use "application/json" when setting frontmatter fields to structured values like arrays (e.g. ["tag1","tag2"]). Defaults to "text/markdown".
    pub content_type: Option<String>,
    /// Content to insert at the target location
    pub content: String,
}

// --- Tool router ---

#[tool_router]
impl ObsidianServer {
    pub fn new(client: Arc<ObsidianClient>) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Read the content of a note at the given path")]
    async fn read_note(
        &self,
        Parameters(args): Parameters<ReadNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        let content = self
            .client
            .read_note(&args.path)
            .await
            .map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    #[tool(description = "Create a new note or overwrite an existing one")]
    async fn create_note(
        &self,
        Parameters(args): Parameters<CreateNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .create_note(&args.path, &args.content)
            .await
            .map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created note at {}",
            args.path
        ))]))
    }

    #[tool(description = "Append content to the end of an existing note")]
    async fn append_note(
        &self,
        Parameters(args): Parameters<AppendNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .append_note(&args.path, &args.content)
            .await
            .map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Appended to {}",
            args.path
        ))]))
    }

    #[tool(
        description = "Partially update a note relative to a heading, block reference, or frontmatter field. For heading targets, sub-headings must use the full path with :: delimiter (e.g. \"Heading 1::Subheading 1\"); only top-level headings can be targeted by name alone."
    )]
    async fn patch_note(
        &self,
        Parameters(args): Parameters<PatchNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        let params = PatchParams {
            operation: args.operation,
            target_type: args.target_type,
            target: args.target,
            target_delimiter: args.target_delimiter,
            trim_target_whitespace: args.trim_target_whitespace,
            create_target_if_missing: args.create_target_if_missing,
            content_type: args.content_type,
        };
        let result = self
            .client
            .patch_note(&args.path, &params, &args.content)
            .await
            .map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Delete a note from the vault")]
    async fn delete_note(
        &self,
        Parameters(args): Parameters<DeleteNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete_note(&args.path)
            .await
            .map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Deleted {}",
            args.path
        ))]))
    }

    #[tool(description = "List files in a vault directory")]
    async fn list_files(
        &self,
        Parameters(args): Parameters<ListFilesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .client
            .list_files(args.path.as_deref())
            .await
            .map_err(to_mcp_error)?;
        let json = serde_json::to_string(&result).map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Search notes by text query")]
    async fn search(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .client
            .search_simple(&args.query)
            .await
            .map_err(to_mcp_error)?;
        let json = serde_json::to_string(&result).map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Search notes using Dataview DQL query. Only TABLE queries are supported (e.g. 'TABLE file.ctime FROM \"folder\"'). LIST and TASK query types are not supported by the Obsidian Local REST API."
    )]
    async fn search_query(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        let trimmed = args.query.trim_start();
        if !trimmed.starts_with("TABLE") {
            return Err(McpError::invalid_params(
                "Only TABLE queries are supported. LIST and TASK query types are not supported by the Obsidian Local REST API.",
                None,
            ));
        }
        let result = self
            .client
            .search_query(&args.query)
            .await
            .map_err(to_mcp_error)?;
        let json = serde_json::to_string(&result).map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "List all available Obsidian commands")]
    async fn list_commands(&self) -> Result<CallToolResult, McpError> {
        let result = self.client.list_commands().await.map_err(to_mcp_error)?;
        let json = serde_json::to_string(&result).map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Execute an Obsidian command by its ID")]
    async fn execute_command(
        &self,
        Parameters(args): Parameters<ExecuteCommandArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .execute_command(&args.command_id)
            .await
            .map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Executed command: {}",
            args.command_id
        ))]))
    }

    #[tool(description = "Open a file in the Obsidian user interface")]
    async fn open_file(
        &self,
        Parameters(args): Parameters<OpenFileArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .open_file(&args.path)
            .await
            .map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Opened {}",
            args.path
        ))]))
    }

    #[tool(description = "Read a periodic note (daily, weekly, monthly, quarterly, yearly)")]
    async fn get_periodic_note(
        &self,
        Parameters(args): Parameters<GetPeriodicNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        let content = self
            .client
            .get_periodic_note(&args.period, args.year, args.month, args.day)
            .await
            .map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    #[tool(description = "Replace the entire content of a periodic note")]
    async fn update_periodic_note(
        &self,
        Parameters(args): Parameters<UpdatePeriodicNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .update_periodic_note(&args.period, args.year, args.month, args.day, &args.content)
            .await
            .map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Updated {} periodic note",
            args.period
        ))]))
    }

    #[tool(description = "Append content to a periodic note")]
    async fn append_periodic_note(
        &self,
        Parameters(args): Parameters<AppendPeriodicNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .append_periodic_note(&args.period, args.year, args.month, args.day, &args.content)
            .await
            .map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Appended to {} periodic note",
            args.period
        ))]))
    }

    #[tool(
        description = "Partially update a periodic note relative to a heading, block reference, or frontmatter field. For heading targets, sub-headings must use the full path with :: delimiter (e.g. \"Heading 1::Subheading 1\"); only top-level headings can be targeted by name alone."
    )]
    async fn patch_periodic_note(
        &self,
        Parameters(args): Parameters<PatchPeriodicNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        let params = PatchParams {
            operation: args.operation,
            target_type: args.target_type,
            target: args.target,
            target_delimiter: args.target_delimiter,
            trim_target_whitespace: args.trim_target_whitespace,
            create_target_if_missing: args.create_target_if_missing,
            content_type: args.content_type,
        };
        let result = self
            .client
            .patch_periodic_note(
                &args.period,
                args.year,
                args.month,
                args.day,
                &params,
                &args.content,
            )
            .await
            .map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get Obsidian Local REST API server status and version info")]
    async fn server_info(&self) -> Result<CallToolResult, McpError> {
        let info = self.client.server_info().await.map_err(to_mcp_error)?;
        let json = serde_json::to_string(&info).map_err(to_mcp_error)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

// --- ServerHandler ---

#[tool_handler]
impl ServerHandler for ObsidianServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "obsidian-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "MCP server for Obsidian vault operations via Local REST API".to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;
    use serde_json::Value;
    use wiremock::matchers::{body_string, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn get_field_description(schema: &schemars::Schema, field: &str) -> String {
        let json: Value = serde_json::to_value(schema).unwrap();
        json["properties"][field]["description"]
            .as_str()
            .unwrap_or("")
            .to_string()
    }

    async fn make_server(mock: &MockServer) -> ObsidianServer {
        let client = ObsidianClient::new(mock.uri(), "test-key".to_string());
        ObsidianServer::new(Arc::new(client))
    }

    fn text_content(result: &CallToolResult) -> &str {
        match &result.content[0].raw {
            RawContent::Text(text) => &text.text,
            _ => panic!("expected text content"),
        }
    }

    #[test]
    fn patch_note_target_field_describes_nested_heading_path() {
        let schema = schema_for!(PatchNoteArgs);
        let desc = get_field_description(&schema, "target");
        assert!(
            desc.contains("Heading 1::Subheading"),
            "PatchNoteArgs 'target' field should document nested heading path with :: syntax, got: {}",
            desc
        );
    }

    #[test]
    fn patch_periodic_note_target_field_describes_nested_heading_path() {
        let schema = schema_for!(PatchPeriodicNoteArgs);
        let desc = get_field_description(&schema, "target");
        assert!(
            desc.contains("Heading 1::Subheading"),
            "PatchPeriodicNoteArgs 'target' field should document nested heading path with :: syntax, got: {}",
            desc
        );
    }

    #[test]
    fn get_info_returns_server_metadata() {
        let client = ObsidianClient::new("https://localhost:27124".to_string(), "k".to_string());
        let server = ObsidianServer::new(Arc::new(client));
        let info = server.get_info();
        assert_eq!(info.server_info.name, "obsidian-mcp");
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
        assert!(info.instructions.is_some());
    }

    #[tokio::test]
    async fn read_note_returns_content() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vault/test.md"))
            .respond_with(ResponseTemplate::new(200).set_body_string("# Hello"))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .read_note(Parameters(ReadNoteArgs {
                path: "test.md".to_string(),
            }))
            .await
            .unwrap();
        assert_eq!(text_content(&result), "# Hello");
    }

    #[tokio::test]
    async fn create_note_returns_confirmation() {
        let mock = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/vault/new.md"))
            .and(body_string("content"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .create_note(Parameters(CreateNoteArgs {
                path: "new.md".to_string(),
                content: "content".to_string(),
            }))
            .await
            .unwrap();
        assert!(text_content(&result).contains("Created note at new.md"));
    }

    #[tokio::test]
    async fn append_note_returns_confirmation() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/vault/note.md"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .append_note(Parameters(AppendNoteArgs {
                path: "note.md".to_string(),
                content: "extra".to_string(),
            }))
            .await
            .unwrap();
        assert!(text_content(&result).contains("Appended to note.md"));
    }

    #[tokio::test]
    async fn patch_note_returns_response() {
        let mock = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/vault/note.md"))
            .respond_with(ResponseTemplate::new(200).set_body_string("patched"))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .patch_note(Parameters(PatchNoteArgs {
                path: "note.md".to_string(),
                operation: Operation::Append,
                target_type: TargetType::Heading,
                target: "Section".to_string(),
                target_delimiter: None,
                trim_target_whitespace: None,
                create_target_if_missing: None,
                content_type: None,
                content: "new text\n".to_string(),
            }))
            .await
            .unwrap();
        assert_eq!(text_content(&result), "patched");
    }

    #[tokio::test]
    async fn delete_note_returns_confirmation() {
        let mock = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/vault/old.md"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .delete_note(Parameters(DeleteNoteArgs {
                path: "old.md".to_string(),
            }))
            .await
            .unwrap();
        assert!(text_content(&result).contains("Deleted old.md"));
    }

    #[tokio::test]
    async fn list_files_returns_json() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vault/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"files": ["a.md", "b.md"]})),
            )
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .list_files(Parameters(ListFilesArgs { path: None }))
            .await
            .unwrap();
        let text = text_content(&result);
        assert!(text.contains("a.md"));
    }

    #[tokio::test]
    async fn search_returns_json() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search/simple/"))
            .and(query_param("query", "test"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([{"filename": "note.md"}])),
            )
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .search(Parameters(SearchArgs {
                query: "test".to_string(),
            }))
            .await
            .unwrap();
        assert!(text_content(&result).contains("note.md"));
    }

    #[tokio::test]
    async fn search_query_rejects_non_table_queries() {
        let mock = MockServer::start().await;
        let server = make_server(&mock).await;
        let err = server
            .search_query(Parameters(SearchArgs {
                query: "LIST FROM \"folder\"".to_string(),
            }))
            .await
            .unwrap_err();
        assert!(err.message.contains("Only TABLE queries"));
    }

    #[tokio::test]
    async fn search_query_accepts_table_query() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .search_query(Parameters(SearchArgs {
                query: "TABLE file.ctime FROM \"folder\"".to_string(),
            }))
            .await
            .unwrap();
        assert_eq!(text_content(&result), "[]");
    }

    #[tokio::test]
    async fn search_query_rejects_task_queries() {
        let mock = MockServer::start().await;
        let server = make_server(&mock).await;
        let err = server
            .search_query(Parameters(SearchArgs {
                query: "TASK FROM \"folder\"".to_string(),
            }))
            .await
            .unwrap_err();
        assert!(err.message.contains("Only TABLE queries"));
    }

    #[tokio::test]
    async fn search_query_rejects_lowercase_table() {
        let mock = MockServer::start().await;
        let server = make_server(&mock).await;
        let err = server
            .search_query(Parameters(SearchArgs {
                query: "table file.ctime FROM \"folder\"".to_string(),
            }))
            .await
            .unwrap_err();
        assert!(err.message.contains("Only TABLE queries"));
    }

    #[tokio::test]
    async fn search_query_accepts_leading_whitespace() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .search_query(Parameters(SearchArgs {
                query: "  TABLE file.ctime FROM \"folder\"".to_string(),
            }))
            .await
            .unwrap();
        assert_eq!(text_content(&result), "[]");
    }

    #[tokio::test]
    async fn list_commands_returns_json() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/commands/"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"commands": []})),
            )
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server.list_commands().await.unwrap();
        assert!(text_content(&result).contains("commands"));
    }

    #[tokio::test]
    async fn execute_command_returns_confirmation() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/commands/app:go-back/"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .execute_command(Parameters(ExecuteCommandArgs {
                command_id: "app:go-back".to_string(),
            }))
            .await
            .unwrap();
        assert!(text_content(&result).contains("Executed command: app:go-back"));
    }

    #[tokio::test]
    async fn open_file_returns_confirmation() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/open/test.md"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .open_file(Parameters(OpenFileArgs {
                path: "test.md".to_string(),
            }))
            .await
            .unwrap();
        assert!(text_content(&result).contains("Opened test.md"));
    }

    #[tokio::test]
    async fn get_periodic_note_returns_content() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/periodic/daily/"))
            .respond_with(ResponseTemplate::new(200).set_body_string("daily content"))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .get_periodic_note(Parameters(GetPeriodicNoteArgs {
                period: "daily".to_string(),
                year: None,
                month: None,
                day: None,
            }))
            .await
            .unwrap();
        assert_eq!(text_content(&result), "daily content");
    }

    #[tokio::test]
    async fn update_periodic_note_returns_confirmation() {
        let mock = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/periodic/daily/"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .update_periodic_note(Parameters(UpdatePeriodicNoteArgs {
                period: "daily".to_string(),
                year: None,
                month: None,
                day: None,
                content: "updated".to_string(),
            }))
            .await
            .unwrap();
        assert!(text_content(&result).contains("Updated daily periodic note"));
    }

    #[tokio::test]
    async fn append_periodic_note_returns_confirmation() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/periodic/daily/"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .append_periodic_note(Parameters(AppendPeriodicNoteArgs {
                period: "daily".to_string(),
                year: None,
                month: None,
                day: None,
                content: "appended".to_string(),
            }))
            .await
            .unwrap();
        assert!(text_content(&result).contains("Appended to daily periodic note"));
    }

    #[tokio::test]
    async fn patch_periodic_note_returns_response() {
        let mock = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/periodic/daily/"))
            .respond_with(ResponseTemplate::new(200).set_body_string("patched periodic"))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server
            .patch_periodic_note(Parameters(PatchPeriodicNoteArgs {
                period: "daily".to_string(),
                year: None,
                month: None,
                day: None,
                operation: Operation::Append,
                target_type: TargetType::Heading,
                target: "Log".to_string(),
                target_delimiter: None,
                trim_target_whitespace: None,
                create_target_if_missing: None,
                content_type: None,
                content: "entry\n".to_string(),
            }))
            .await
            .unwrap();
        assert_eq!(text_content(&result), "patched periodic");
    }

    #[tokio::test]
    async fn server_info_returns_json() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "OK",
                "versions": {}
            })))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let result = server.server_info().await.unwrap();
        assert!(text_content(&result).contains("OK"));
    }

    #[tokio::test]
    async fn tool_returns_error_on_api_failure() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vault/missing.md"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let err = server
            .read_note(Parameters(ReadNoteArgs {
                path: "missing.md".to_string(),
            }))
            .await
            .unwrap_err();
        assert!(err.message.contains("404"));
    }

    #[tokio::test]
    async fn tool_returns_error_on_500() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vault/note.md"))
            .respond_with(
                ResponseTemplate::new(500).set_body_string("internal server error"),
            )
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let err = server
            .read_note(Parameters(ReadNoteArgs {
                path: "note.md".to_string(),
            }))
            .await
            .unwrap_err();
        assert!(err.message.contains("500"));
    }

    #[tokio::test]
    async fn tool_returns_error_on_malformed_json() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vault/"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("this is not json"),
            )
            .mount(&mock)
            .await;

        let server = make_server(&mock).await;
        let err = server
            .list_files(Parameters(ListFilesArgs { path: None }))
            .await
            .unwrap_err();
        assert!(
            !err.message.is_empty(),
            "expected a non-empty error message for malformed JSON"
        );
    }
}
