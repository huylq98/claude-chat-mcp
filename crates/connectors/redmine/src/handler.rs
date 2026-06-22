//! The Redmine MCP server: one stdio server exposing Redmine tools.

use crate::client::RedmineClient;
use crate::config::Config;
use crate::format;
use rmcp::schemars::JsonSchema;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use std::sync::Arc;

// ---- Tool argument types -----------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListProjectsArgs {
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListIssuesArgs {
    /// Numeric project id or project identifier to filter by (optional).
    pub project_id: Option<String>,
    /// Issue status: `open`, `closed`, or `*` for all (default `open`).
    pub status: Option<String>,
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetIssueArgs {
    /// The issue id.
    pub issue_id: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateIssueArgs {
    /// Numeric project id or project identifier.
    pub project_id: String,
    /// Issue subject (title).
    pub subject: String,
    /// Optional issue description.
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddNoteArgs {
    /// The issue id.
    pub issue_id: u64,
    /// Note text to add to the issue.
    pub note: String,
}

// ---- Server ------------------------------------------------------------------

#[derive(Clone)]
pub struct Server {
    client: Arc<RedmineClient>,
    config: Arc<Config>,
    tool_router: ToolRouter<Server>,
}

fn ok(text: String) -> Result<CallToolResult, rmcp::ErrorData> {
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

#[tool_router]
impl Server {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = Config::from_env();
        config.validate()?;
        let client = RedmineClient::from_config(&config)?;
        // Viewer mode strips the write tools so Claude only sees read-only ones.
        let mut tool_router = Self::tool_router();
        if connector_core::Mode::from_env("REDMINE_MODE").is_viewer() {
            for write_tool in ["redmine_create_issue", "redmine_add_note"] {
                tool_router.remove_route(write_tool);
            }
        }
        Ok(Self {
            client: Arc::new(client),
            config: Arc::new(config),
            tool_router,
        })
    }

    fn max_len(&self) -> usize {
        self.config.max_content_length
    }

    #[tool(description = "List Redmine projects.")]
    async fn redmine_list_projects(
        &self,
        Parameters(args): Parameters<ListProjectsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50);
        let text = match self.client.list_projects(limit).await {
            Ok(d) => format::projects(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "List issues in Redmine (status: open|closed|*), optionally filtered by project.")]
    async fn redmine_list_issues(
        &self,
        Parameters(args): Parameters<ListIssuesArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50);
        let status = args.status.as_deref().unwrap_or("open");
        let text = match self
            .client
            .list_issues(args.project_id.as_deref(), status, limit)
            .await
        {
            Ok(d) => format::issues(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Get a single Redmine issue by its id.")]
    async fn redmine_get_issue(
        &self,
        Parameters(args): Parameters<GetIssueArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.get_issue(args.issue_id).await {
            Ok(d) => format::issue(&d, self.max_len()),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Create a new issue in a Redmine project.")]
    async fn redmine_create_issue(
        &self,
        Parameters(args): Parameters<CreateIssueArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let description = args.description.as_deref().unwrap_or("");
        let text = match self
            .client
            .create_issue(&args.project_id, &args.subject, description)
            .await
        {
            Ok(d) => format::created_issue(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Add a note (comment) to a Redmine issue.")]
    async fn redmine_add_note(
        &self,
        Parameters(args): Parameters<AddNoteArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        // PUT returns 204 with no body; surface a synthetic confirmation.
        let text = match self.client.add_note(args.issue_id, &args.note).await {
            Ok(_) => format::added_note(args.issue_id),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }
}

impl Server {
    /// Make one cheap authenticated call to verify the connection works.
    /// Used by the `--test-connection` binary mode.
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        self.client.current_user().await?;
        Ok(())
    }
}

#[tool_handler]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "redmine".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Self-hosted Redmine integration. Use the redmine_* tools to list projects, \
                 list and read issues. In Writer mode you can also create issues and add notes \
                 to them. The project_id argument accepts either a numeric project id or a \
                 project identifier."
                    .to_string(),
            ),
        }
    }
}
