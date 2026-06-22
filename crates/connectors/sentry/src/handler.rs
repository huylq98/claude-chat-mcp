//! The Sentry MCP server: one stdio server exposing Sentry tools.

use crate::client::SentryClient;
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
pub struct ListProjectsArgs {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListIssuesArgs {
    /// Organization slug that owns the project.
    pub org_slug: String,
    /// Project slug to list issues for.
    pub project_slug: String,
    /// Optional Sentry search query (e.g. `is:unresolved`).
    pub query: Option<String>,
    /// Max results (1-100, default 25).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetIssueArgs {
    /// The Sentry issue id.
    pub issue_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateIssueStatusArgs {
    /// The Sentry issue id.
    pub issue_id: String,
    /// New status: `resolved`, `ignored`, or `unresolved`.
    pub status: String,
}

// ---- Server ------------------------------------------------------------------

#[derive(Clone)]
pub struct Server {
    client: Arc<SentryClient>,
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
        let client = SentryClient::from_config(&config)?;
        // Viewer mode strips the write tools so Claude only sees read-only ones.
        let mut tool_router = Self::tool_router();
        if connector_core::Mode::from_env("SENTRY_MODE").is_viewer() {
            tool_router.remove_route("sentry_update_issue_status");
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

    #[tool(description = "List Sentry projects you can access.")]
    async fn sentry_list_projects(
        &self,
        Parameters(_args): Parameters<ListProjectsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.list_projects().await {
            Ok(d) => format::projects(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "List issues in a Sentry project (optional Sentry search query).")]
    async fn sentry_list_issues(
        &self,
        Parameters(args): Parameters<ListIssuesArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(25).clamp(1, 100);
        let query = args.query.as_deref().unwrap_or("");
        let text = match self
            .client
            .list_issues(&args.org_slug, &args.project_slug, query, limit)
            .await
        {
            Ok(d) => format::issues(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Get a single Sentry issue by its id.")]
    async fn sentry_get_issue(
        &self,
        Parameters(args): Parameters<GetIssueArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.get_issue(&args.issue_id).await {
            Ok(d) => format::issue(&d, self.max_len()),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(
        description = "Update a Sentry issue's status (resolved|ignored|unresolved)."
    )]
    async fn sentry_update_issue_status(
        &self,
        Parameters(args): Parameters<UpdateIssueStatusArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self
            .client
            .update_issue_status(&args.issue_id, &args.status)
            .await
        {
            Ok(d) => format::updated_status(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }
}

impl Server {
    /// Make one cheap authenticated call to verify the connection works.
    /// Used by the `--test-connection` binary mode.
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        self.client.ping().await?;
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
                name: "sentry".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Self-hosted Sentry integration. Use the sentry_* tools to list projects, \
                 list and read issues. In Writer mode you can also update an issue's status \
                 (resolve, ignore, or unresolve it). List issues with the org_slug and \
                 project_slug from sentry_list_projects."
                    .to_string(),
            ),
        }
    }
}
