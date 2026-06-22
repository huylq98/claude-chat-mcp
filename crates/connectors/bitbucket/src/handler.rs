//! The Bitbucket MCP server: one stdio server exposing Bitbucket tools.

use crate::client::BitbucketClient;
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
pub struct ReposArgs {
    /// The Bitbucket project key, e.g. `ENG`.
    pub project_key: String,
    /// Max results (default 25).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrArgs {
    /// The Bitbucket project key.
    pub project_key: String,
    /// The repository slug.
    pub repo_slug: String,
    /// PR state: OPEN, MERGED, DECLINED, or ALL. Default OPEN.
    pub state: Option<String>,
    /// Max results (default 25).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommitsArgs {
    /// The Bitbucket project key.
    pub project_key: String,
    /// The repository slug.
    pub repo_slug: String,
    /// Max results (default 25).
    pub limit: Option<u32>,
}

// ---- Server ------------------------------------------------------------------

#[derive(Clone)]
pub struct Server {
    client: Arc<BitbucketClient>,
    #[allow(dead_code)]
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
        let client = BitbucketClient::from_config(&config)?;
        Ok(Self {
            client: Arc::new(client),
            config: Arc::new(config),
            tool_router: Self::tool_router(),
        })
    }

    #[tool(description = "List repositories in a Bitbucket project.")]
    async fn bitbucket_list_repos(
        &self,
        Parameters(args): Parameters<ReposArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(25);
        let text = match self.client.list_repos(&args.project_key, limit).await {
            Ok(d) => format::repos(&d),
            Err(e) => format::error(&e),
        };
        ok(text)
    }

    #[tool(description = "List pull requests for a Bitbucket repository.")]
    async fn bitbucket_list_pull_requests(
        &self,
        Parameters(args): Parameters<PrArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(25);
        let state = args.state.as_deref().unwrap_or("OPEN").to_uppercase();
        let text = match self
            .client
            .list_pull_requests(&args.project_key, &args.repo_slug, &state, limit)
            .await
        {
            Ok(d) => format::pull_requests(&d),
            Err(e) => format::error(&e),
        };
        ok(text)
    }

    #[tool(description = "List recent commits on a Bitbucket repository's default branch.")]
    async fn bitbucket_get_commits(
        &self,
        Parameters(args): Parameters<CommitsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(25);
        let text = match self
            .client
            .get_commits(&args.project_key, &args.repo_slug, limit)
            .await
        {
            Ok(d) => format::commits(&d),
            Err(e) => format::error(&e),
        };
        ok(text)
    }
}

#[tool_handler]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "bitbucket".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Self-hosted Bitbucket integration (Server / Data Center). Use the bitbucket_* \
                 tools to list repositories in a project, list a repository's pull requests, and \
                 list recent commits."
                    .to_string(),
            ),
        }
    }
}
