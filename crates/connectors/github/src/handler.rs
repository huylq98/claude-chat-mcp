//! The GitHub MCP server: one stdio server exposing GitHub tools.

use crate::client::GitHubClient;
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
pub struct SearchReposArgs {
    /// Search query (GitHub search syntax) for repository names and metadata.
    pub query: String,
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListIssuesArgs {
    /// Repository owner (user or organization login).
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Issue state: `open`, `closed`, or `all` (default `open`).
    pub state: Option<String>,
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetIssueArgs {
    /// Repository owner (user or organization login).
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// The issue number within the repository.
    pub number: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListPullRequestsArgs {
    /// Repository owner (user or organization login).
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// PR state: `open`, `closed`, or `all` (default `open`).
    pub state: Option<String>,
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateIssueArgs {
    /// Repository owner (user or organization login).
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Issue title.
    pub title: String,
    /// Optional issue body (markdown).
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommentIssueArgs {
    /// Repository owner (user or organization login).
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// The issue number within the repository.
    pub number: u64,
    /// Comment body (markdown).
    pub body: String,
}

// ---- Server ------------------------------------------------------------------

#[derive(Clone)]
pub struct Server {
    client: Arc<GitHubClient>,
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
        let client = GitHubClient::from_config(&config)?;
        // Viewer mode strips the write tools so Claude only sees read-only ones.
        let mut tool_router = Self::tool_router();
        if connector_core::Mode::from_env("GITHUB_MODE").is_viewer() {
            for write_tool in ["github_create_issue", "github_comment_issue"] {
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

    #[tool(description = "Search GitHub repositories by name or metadata.")]
    async fn github_search_repos(
        &self,
        Parameters(args): Parameters<SearchReposArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50);
        let text = match self.client.search_repos(&args.query, limit).await {
            Ok(d) => format::repos(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "List issues in a GitHub repository (state: open|closed|all).")]
    async fn github_list_issues(
        &self,
        Parameters(args): Parameters<ListIssuesArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50);
        let state = args.state.as_deref().unwrap_or("open");
        let text = match self
            .client
            .list_issues(&args.owner, &args.repo, state, limit)
            .await
        {
            Ok(d) => format::issues(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Get a single GitHub issue by its number.")]
    async fn github_get_issue(
        &self,
        Parameters(args): Parameters<GetIssueArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self
            .client
            .get_issue(&args.owner, &args.repo, args.number)
            .await
        {
            Ok(d) => format::issue(&d, self.max_len()),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "List pull requests in a GitHub repository (state: open|closed|all).")]
    async fn github_list_pull_requests(
        &self,
        Parameters(args): Parameters<ListPullRequestsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50);
        let state = args.state.as_deref().unwrap_or("open");
        let text = match self
            .client
            .list_pull_requests(&args.owner, &args.repo, state, limit)
            .await
        {
            Ok(d) => format::pull_requests(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Create a new issue in a GitHub repository.")]
    async fn github_create_issue(
        &self,
        Parameters(args): Parameters<CreateIssueArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let body = args.body.as_deref().unwrap_or("");
        let text = match self
            .client
            .create_issue(&args.owner, &args.repo, &args.title, body)
            .await
        {
            Ok(d) => format::created_issue(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Add a comment to a GitHub issue.")]
    async fn github_comment_issue(
        &self,
        Parameters(args): Parameters<CommentIssueArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self
            .client
            .comment_issue(&args.owner, &args.repo, args.number, &args.body)
            .await
        {
            Ok(d) => format::created_comment(&d),
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
                name: "github".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Self-hosted GitHub (Enterprise Server) integration. Use the github_* tools \
                 to search repositories, list and read issues, and list pull requests. In \
                 Writer mode you can also create issues and comment on them. Repositories are \
                 addressed by their owner and repo name."
                    .to_string(),
            ),
        }
    }
}
