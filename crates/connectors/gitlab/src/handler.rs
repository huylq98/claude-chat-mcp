//! The GitLab MCP server: one stdio server exposing GitLab tools.

use crate::client::GitLabClient;
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
pub struct SearchProjectsArgs {
    /// Term to search project names and paths for.
    pub search: String,
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListIssuesArgs {
    /// Numeric project id or URL-encoded `group/project` path.
    pub project_id: String,
    /// Issue state: `opened`, `closed`, or `all` (default `opened`).
    pub state: Option<String>,
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetIssueArgs {
    /// Numeric project id or `group/project` path.
    pub project_id: String,
    /// The issue internal id (iid) within the project.
    pub issue_iid: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListMergeRequestsArgs {
    /// Numeric project id or `group/project` path.
    pub project_id: String,
    /// MR state: `opened`, `closed`, `merged`, or `all` (default `opened`).
    pub state: Option<String>,
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateIssueArgs {
    /// Numeric project id or `group/project` path.
    pub project_id: String,
    /// Issue title.
    pub title: String,
    /// Optional issue description (markdown).
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommentIssueArgs {
    /// Numeric project id or `group/project` path.
    pub project_id: String,
    /// The issue internal id (iid) within the project.
    pub issue_iid: u64,
    /// Comment body (markdown).
    pub body: String,
}

// ---- Server ------------------------------------------------------------------

#[derive(Clone)]
pub struct Server {
    client: Arc<GitLabClient>,
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
        let client = GitLabClient::from_config(&config)?;
        // Viewer mode strips the write tools so Claude only sees read-only ones.
        let mut tool_router = Self::tool_router();
        if connector_core::Mode::from_env("GITLAB_MODE").is_viewer() {
            for write_tool in ["gitlab_create_issue", "gitlab_comment_issue"] {
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

    #[tool(description = "Search GitLab projects you are a member of by name or path.")]
    async fn gitlab_search_projects(
        &self,
        Parameters(args): Parameters<SearchProjectsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50);
        let text = match self.client.search_projects(&args.search, limit).await {
            Ok(d) => format::projects(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "List issues in a GitLab project (state: opened|closed|all).")]
    async fn gitlab_list_issues(
        &self,
        Parameters(args): Parameters<ListIssuesArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50);
        let state = args.state.as_deref().unwrap_or("opened");
        let text = match self.client.list_issues(&args.project_id, state, limit).await {
            Ok(d) => format::issues(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Get a single GitLab issue by its project-internal id (iid).")]
    async fn gitlab_get_issue(
        &self,
        Parameters(args): Parameters<GetIssueArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.get_issue(&args.project_id, args.issue_iid).await {
            Ok(d) => format::issue(&d, self.max_len()),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "List merge requests in a GitLab project (state: opened|closed|merged|all).")]
    async fn gitlab_list_merge_requests(
        &self,
        Parameters(args): Parameters<ListMergeRequestsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50);
        let state = args.state.as_deref().unwrap_or("opened");
        let text = match self
            .client
            .list_merge_requests(&args.project_id, state, limit)
            .await
        {
            Ok(d) => format::merge_requests(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Create a new issue in a GitLab project.")]
    async fn gitlab_create_issue(
        &self,
        Parameters(args): Parameters<CreateIssueArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let description = args.description.as_deref().unwrap_or("");
        let text = match self
            .client
            .create_issue(&args.project_id, &args.title, description)
            .await
        {
            Ok(d) => format::created_issue(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Add a comment to a GitLab issue.")]
    async fn gitlab_comment_issue(
        &self,
        Parameters(args): Parameters<CommentIssueArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self
            .client
            .comment_issue(&args.project_id, args.issue_iid, &args.body)
            .await
        {
            Ok(d) => format::created_note(&d),
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
                name: "gitlab".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Self-hosted GitLab integration. Use the gitlab_* tools to search projects, \
                 list and read issues, and list merge requests. In Writer mode you can also \
                 create issues and comment on them. The project_id argument accepts either a \
                 numeric project id or a 'group/project' path."
                    .to_string(),
            ),
        }
    }
}
