//! The Jira MCP server: one stdio server exposing Jira tools.

use crate::client::JiraClient;
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
pub struct SearchArgs {
    /// A JQL query, e.g. `project = ENG AND status = "In Progress"`.
    pub jql: String,
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueArgs {
    /// The issue key, e.g. `ENG-1234`.
    pub issue_key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateIssueArgs {
    /// The project key the issue belongs to, e.g. `ENG`.
    pub project_key: String,
    /// The issue summary (title).
    pub summary: String,
    /// The issue description.
    pub description: String,
    /// The issue type, e.g. `Task`, `Bug`, `Story`. Default `Task`.
    pub issue_type: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddCommentArgs {
    /// The issue key to comment on, e.g. `ENG-1234`.
    pub issue_key: String,
    /// The comment body.
    pub body: String,
}

// ---- Server ------------------------------------------------------------------

#[derive(Clone)]
pub struct Server {
    client: Arc<JiraClient>,
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
        let client = JiraClient::from_config(&config)?;
        // Viewer mode strips the write tools so Claude only sees read-only ones.
        let mut tool_router = Self::tool_router();
        if connector_core::Mode::from_env("JIRA_MODE").is_viewer() {
            for write_tool in ["jira_create_issue", "jira_add_comment"] {
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

    #[tool(description = "Search Jira (self-hosted) issues using JQL.")]
    async fn jira_search(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50);
        let text = match self.client.search(&args.jql, limit).await {
            Ok(d) => format::search(&d),
            Err(e) => format::error(&e),
        };
        ok(text)
    }

    #[tool(description = "Get a single Jira issue by key (e.g. ENG-1234).")]
    async fn jira_get_issue(
        &self,
        Parameters(args): Parameters<IssueArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.get_issue(&args.issue_key).await {
            Ok(d) => format::issue(&d, self.max_len()),
            Err(e) => format::error(&e),
        };
        ok(text)
    }

    #[tool(description = "List Jira projects.")]
    async fn jira_list_projects(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.list_projects().await {
            Ok(d) => format::projects(&d),
            Err(e) => format::error(&e),
        };
        ok(text)
    }

    #[tool(description = "Create a new Jira issue in a project (Writer mode).")]
    async fn jira_create_issue(
        &self,
        Parameters(args): Parameters<CreateIssueArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let issue_type = args.issue_type.as_deref().unwrap_or("Task");
        let text = match self
            .client
            .create_issue(&args.project_key, &args.summary, &args.description, issue_type)
            .await
        {
            Ok(d) => format::created_issue(&d),
            Err(e) => format::error(&e),
        };
        ok(text)
    }

    #[tool(description = "Add a comment to a Jira issue (Writer mode).")]
    async fn jira_add_comment(
        &self,
        Parameters(args): Parameters<AddCommentArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.add_comment(&args.issue_key, &args.body).await {
            Ok(d) => format::created_comment(&d),
            Err(e) => format::error(&e),
        };
        ok(text)
    }
}

impl Server {
    /// Make one cheap authenticated call to verify the connection works.
    /// Used by the `--test-connection` binary mode.
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        self.client.list_projects().await?;
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
                name: "jira".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Self-hosted Jira integration (Data Center / Server). Use the jira_* tools to \
                 search issues with JQL, read an issue by key, and list projects."
                    .to_string(),
            ),
        }
    }
}
