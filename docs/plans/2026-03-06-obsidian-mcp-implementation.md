# Obsidian MCP Server Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust MCP server that exposes 16 Obsidian vault tools via the Local REST API plugin.

**Architecture:** Single binary with three layers — CLI config (clap), MCP server (rmcp with Streamable HTTP), and ObsidianClient (reqwest). All tools call ObsidianClient methods which forward to the Local REST API.

**Tech Stack:** Rust, rmcp (MCP SDK), reqwest, clap, axum, tokio, serde, schemars, thiserror

**Reference:** See `docs/plans/2026-03-06-obsidian-mcp-design.md` for full design document.

---

### Task 1: Project scaffolding and Cargo.toml

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "obsidian-mcp"
version = "0.1.0"
edition = "2024"

[dependencies]
rmcp = { version = "0.12", features = [
    "server",
    "macros",
    "transport-streamable-http-server",
] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
clap = { version = "4", features = ["derive", "env"] }
tokio = { version = "1", features = ["full"] }
axum = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**Step 2: Create minimal main.rs**

```rust
fn main() {
    println!("obsidian-mcp");
}
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds (dependencies download and compile)

**Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "feat: scaffold project with dependencies"
```

---

### Task 2: Error type

**Files:**
- Create: `src/error.rs`
- Modify: `src/main.rs` (add `mod error;`)

**Step 1: Create error.rs**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Obsidian API error ({status}): {body}")]
    Api { status: u16, body: String },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
```

**Step 2: Add module to main.rs**

```rust
mod error;

fn main() {
    println!("obsidian-mcp");
}
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/error.rs src/main.rs
git commit -m "feat: add unified error type"
```

---

### Task 3: ObsidianClient — constructor and server_info

**Files:**
- Create: `src/client.rs`
- Modify: `src/main.rs` (add `mod client;`)

**Step 1: Create client.rs with constructor and server_info**

```rust
use reqwest::Client;
use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct ServerInfo {
    pub status: String,
    #[serde(default)]
    pub versions: serde_json::Value,
}

pub struct ObsidianClient {
    http: Client,
    base_url: String,
    api_key: String,
}

impl ObsidianClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        let http = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("failed to build HTTP client");

        Self {
            http,
            base_url,
            api_key,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    pub async fn server_info(&self) -> Result<ServerInfo, AppError> {
        let resp = self
            .http
            .get(self.url("/"))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.json().await?)
    }
}
```

**Step 2: Add module to main.rs**

```rust
mod client;
mod error;

fn main() {
    println!("obsidian-mcp");
}
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/client.rs src/main.rs
git commit -m "feat: add ObsidianClient with server_info"
```

---

### Task 4: ObsidianClient — vault file operations

**Files:**
- Modify: `src/client.rs`

**Step 1: Add vault file methods**

Append to `impl ObsidianClient`:

```rust
    pub async fn read_note(&self, path: &str) -> Result<String, AppError> {
        let resp = self
            .http
            .get(self.url(&format!("/vault/{}", path)))
            .header("Authorization", self.auth_header())
            .header("Accept", "text/markdown")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.text().await?)
    }

    pub async fn create_note(&self, path: &str, content: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .put(self.url(&format!("/vault/{}", path)))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    pub async fn append_note(&self, path: &str, content: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .post(self.url(&format!("/vault/{}", path)))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    pub async fn patch_note(
        &self,
        path: &str,
        heading: Option<&str>,
        content: &str,
    ) -> Result<String, AppError> {
        let mut req = self
            .http
            .patch(self.url(&format!("/vault/{}", path)))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown");

        if let Some(heading) = heading {
            req = req.header("X-Heading", heading);
        }

        let resp = req.body(content.to_string()).send().await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.text().await?)
    }

    pub async fn delete_note(&self, path: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .delete(self.url(&format!("/vault/{}", path)))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    pub async fn list_files(&self, path: Option<&str>) -> Result<serde_json::Value, AppError> {
        let url = match path {
            Some(p) => self.url(&format!("/vault/{}/", p)),
            None => self.url("/vault/"),
        };

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.json().await?)
    }
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/client.rs
git commit -m "feat: add vault file operations to ObsidianClient"
```

---

### Task 5: ObsidianClient — search, commands, open

**Files:**
- Modify: `src/client.rs`

**Step 1: Add search, commands, and open methods**

Append to `impl ObsidianClient`:

```rust
    pub async fn search_simple(&self, query: &str) -> Result<serde_json::Value, AppError> {
        let resp = self
            .http
            .post(self.url("/search/simple/"))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/plain")
            .body(query.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.json().await?)
    }

    pub async fn search_query(&self, query: &str) -> Result<serde_json::Value, AppError> {
        let resp = self
            .http
            .post(self.url("/search/"))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/vnd.olrapi.dataview.dql+txt")
            .body(query.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.json().await?)
    }

    pub async fn list_commands(&self) -> Result<serde_json::Value, AppError> {
        let resp = self
            .http
            .get(self.url("/commands/"))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.json().await?)
    }

    pub async fn execute_command(&self, command_id: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .post(self.url(&format!("/commands/{}/", command_id)))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    pub async fn open_file(&self, filename: &str) -> Result<(), AppError> {
        let resp = self
            .http
            .post(self.url(&format!("/open/{}", filename)))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/client.rs
git commit -m "feat: add search, commands, and open to ObsidianClient"
```

---

### Task 6: ObsidianClient — periodic notes

**Files:**
- Modify: `src/client.rs`

**Step 1: Add periodic note helper and methods**

Append to `impl ObsidianClient`:

```rust
    fn periodic_url(
        &self,
        period: &str,
        year: Option<u32>,
        month: Option<u32>,
        day: Option<u32>,
    ) -> String {
        match (year, month, day) {
            (Some(y), Some(m), Some(d)) => {
                self.url(&format!("/periodic/{}/{}/{}/{}/", period, y, m, d))
            }
            _ => self.url(&format!("/periodic/{}/", period)),
        }
    }

    pub async fn get_periodic_note(
        &self,
        period: &str,
        year: Option<u32>,
        month: Option<u32>,
        day: Option<u32>,
    ) -> Result<String, AppError> {
        let resp = self
            .http
            .get(self.periodic_url(period, year, month, day))
            .header("Authorization", self.auth_header())
            .header("Accept", "text/markdown")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.text().await?)
    }

    pub async fn update_periodic_note(
        &self,
        period: &str,
        year: Option<u32>,
        month: Option<u32>,
        day: Option<u32>,
        content: &str,
    ) -> Result<(), AppError> {
        let resp = self
            .http
            .put(self.periodic_url(period, year, month, day))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    pub async fn append_periodic_note(
        &self,
        period: &str,
        year: Option<u32>,
        month: Option<u32>,
        day: Option<u32>,
        content: &str,
    ) -> Result<(), AppError> {
        let resp = self
            .http
            .post(self.periodic_url(period, year, month, day))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    pub async fn patch_periodic_note(
        &self,
        period: &str,
        year: Option<u32>,
        month: Option<u32>,
        day: Option<u32>,
        heading: Option<&str>,
        content: &str,
    ) -> Result<String, AppError> {
        let mut req = self
            .http
            .patch(self.periodic_url(period, year, month, day))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown");

        if let Some(heading) = heading {
            req = req.header("X-Heading", heading);
        }

        let resp = req.body(content.to_string()).send().await?;

        if !resp.status().is_success() {
            return Err(AppError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.text().await?)
    }
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/client.rs
git commit -m "feat: add periodic note operations to ObsidianClient"
```

---

### Task 7: MCP Server — tool handlers (vault files + search)

**Files:**
- Create: `src/server.rs`
- Modify: `src/main.rs` (add `mod server;`)

**Step 1: Create server.rs with vault file and search tools**

```rust
use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
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
        let content = self.client.read_note(&args.path).await.map_err(|e| {
            McpError::internal_error(e.to_string(), None)
        })?;
        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    #[tool(description = "Create a new note or overwrite an existing one")]
    async fn create_note(
        &self,
        Parameters(args): Parameters<CreateNoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.client.create_note(&args.path, &args.content).await.map_err(|e| {
            McpError::internal_error(e.to_string(), None)
        })?;
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
        self.client.append_note(&args.path, &args.content).await.map_err(|e| {
            McpError::internal_error(e.to_string(), None)
        })?;
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
        self.client.delete_note(&args.path).await.map_err(|e| {
            McpError::internal_error(e.to_string(), None)
        })?;
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
}
```

**Step 2: Add module to main.rs**

```rust
mod client;
mod error;
mod server;

fn main() {
    println!("obsidian-mcp");
}
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/server.rs src/main.rs
git commit -m "feat: add MCP tool handlers for vault files and search"
```

---

### Task 8: MCP Server — tool handlers (commands, open, periodic, system)

**Files:**
- Modify: `src/server.rs`

**Step 1: Add remaining arg structs and tool handlers**

Add these structs before the `#[tool_router] impl`:

```rust
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
```

Add these tool handlers inside the `#[tool_router] impl ObsidianServer`:

```rust
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
            .update_periodic_note(
                &args.period,
                args.year,
                args.month,
                args.day,
                &args.content,
            )
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
            .append_periodic_note(
                &args.period,
                args.year,
                args.month,
                args.day,
                &args.content,
            )
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
```

**Step 2: Add ServerHandler implementation**

Add after the `#[tool_router] impl` block:

```rust
#[tool_handler]
impl ServerHandler for ObsidianServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "obsidian-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "MCP server for Obsidian vault operations via Local REST API".to_string(),
            ),
        }
    }
}
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/server.rs
git commit -m "feat: add remaining tool handlers and ServerHandler impl"
```

---

### Task 9: CLI config and main startup

**Files:**
- Modify: `src/main.rs`

**Step 1: Implement full main.rs**

```rust
mod client;
mod error;
mod server;

use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use rmcp::transport::streamable_http_server::tower::{
    StreamableHttpServerConfig, StreamableHttpService,
};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use tokio_util::sync::CancellationToken;

use crate::client::ObsidianClient;
use crate::server::ObsidianServer;

#[derive(Parser)]
#[command(name = "obsidian-mcp", about = "MCP server for Obsidian vault operations")]
struct Cli {
    /// Obsidian REST API URL
    #[arg(long, env = "OBSIDIAN_API_URL", default_value = "https://127.0.0.1:27124")]
    api_url: String,

    /// Obsidian REST API key
    #[arg(long, env = "OBSIDIAN_API_KEY")]
    api_key: String,

    /// MCP server listen port
    #[arg(long, env = "MCP_PORT", default_value = "3000")]
    port: u16,

    /// MCP server listen host
    #[arg(long, env = "MCP_HOST", default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("obsidian_mcp=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    // Create Obsidian API client
    let client = Arc::new(ObsidianClient::new(cli.api_url.clone(), cli.api_key));

    // Verify connection to Obsidian
    tracing::info!("Connecting to Obsidian at {}...", cli.api_url);
    match client.server_info().await {
        Ok(info) => tracing::info!("Connected to Obsidian: {:?}", info),
        Err(e) => {
            tracing::error!("Failed to connect to Obsidian: {}", e);
            std::process::exit(1);
        }
    }

    // Set up MCP server
    let cancel_token = CancellationToken::new();
    let config = StreamableHttpServerConfig {
        stateful_mode: true,
        cancellation_token: cancel_token.clone(),
        ..Default::default()
    };

    let session_manager = Arc::new(LocalSessionManager::default());
    let client_clone = client.clone();
    let service = StreamableHttpService::new(
        move || Ok(ObsidianServer::new(client_clone.clone())),
        session_manager,
        config,
    );

    let app = axum::Router::new().nest_service("/mcp", service);

    let addr: SocketAddr = format!("{}:{}", cli.host, cli.port).parse()?;
    tracing::info!("MCP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("Shutting down...");
            cancel_token.cancel();
        })
        .await?;

    Ok(())
}
```

**Step 2: Add anyhow and tokio-util to Cargo.toml**

Add to `[dependencies]`:

```toml
anyhow = "1"
tokio-util = "0.7"
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/main.rs Cargo.toml Cargo.lock
git commit -m "feat: add CLI config and server startup"
```

---

### Task 10: Verify build and do a smoke test

**Files:** (none — verification only)

**Step 1: Run cargo clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings or errors

**Step 2: Run cargo fmt**

Run: `cargo fmt`
Expected: Code is formatted

**Step 3: Commit any formatting fixes**

```bash
git add -A
git commit -m "style: apply rustfmt formatting"
```

(Skip if no changes.)

**Step 4: Try running the binary**

Run: `cargo run -- --api-key test-key --api-url https://127.0.0.1:27124`
Expected: Should print connection error (no Obsidian running) and exit with error code. This confirms the startup flow works correctly.

---

### Task 11: Update .gitignore and README

**Files:**
- Modify: `.gitignore`
- Modify: `README.md`

**Step 1: Update .gitignore**

Add standard Rust entries:

```gitignore
/target
```

**Step 2: Update README.md**

```markdown
# obsidian-mcp

An MCP (Model Context Protocol) server that exposes Obsidian vault operations as tools for AI assistants. Communicates with Obsidian through the [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) plugin.

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
| `patch_note` | Partially update a note relative to a heading |
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
| `patch_periodic_note` | Partially update a periodic note |
| `server_info` | Get Obsidian API server status |

## License

MIT
```

**Step 3: Commit**

```bash
git add .gitignore README.md
git commit -m "docs: update README and gitignore"
```
