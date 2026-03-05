use std::sync::Arc;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::ObsidianClient;

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
    /// Target heading to patch under (optional)
    pub heading: Option<String>,
    /// New content for the target section
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
    /// Target heading to patch under (optional)
    pub heading: Option<String>,
    /// New content for the target section
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
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
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
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
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
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Appended to {}",
            args.path
        ))]))
    }

    #[tool(description = "Partially update a note relative to a heading or frontmatter field")]
    async fn patch_note(
        &self,
        Parameters(args): Parameters<PatchNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .client
            .patch_note(&args.path, args.heading.as_deref(), &args.content)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
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
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
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
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
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
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Search notes using Dataview DQL query")]
    async fn search_query(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .client
            .search_query(&args.query)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "List all available Obsidian commands")]
    async fn list_commands(&self) -> Result<CallToolResult, McpError> {
        let result = self
            .client
            .list_commands()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
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
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
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
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
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
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
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
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
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
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Appended to {} periodic note",
            args.period
        ))]))
    }

    #[tool(description = "Partially update a periodic note relative to a heading")]
    async fn patch_periodic_note(
        &self,
        Parameters(args): Parameters<PatchPeriodicNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .client
            .patch_periodic_note(
                &args.period,
                args.year,
                args.month,
                args.day,
                args.heading.as_deref(),
                &args.content,
            )
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get Obsidian Local REST API server status and version info")]
    async fn server_info(&self) -> Result<CallToolResult, McpError> {
        let info = self
            .client
            .server_info()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let json = serde_json::to_string_pretty(&info)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
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
