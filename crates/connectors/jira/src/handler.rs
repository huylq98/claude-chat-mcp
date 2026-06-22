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
        Ok(Self {
            client: Arc::new(client),
            config: Arc::new(config),
            tool_router: Self::tool_router(),
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
