//! The Grafana MCP server: one stdio server exposing Grafana tools.

use crate::client::GrafanaClient;
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
pub struct SearchDashboardsArgs {
    /// Term to match dashboard titles against (empty matches all).
    pub query: Option<String>,
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDashboardArgs {
    /// The dashboard uid.
    pub uid: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAlertsArgs {
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateAnnotationArgs {
    /// Annotation text (markdown).
    pub text: String,
    /// Optional tags to attach to the annotation.
    pub tags: Option<Vec<String>>,
    /// Optional dashboard uid to scope the annotation to.
    pub dashboard_uid: Option<String>,
}

// ---- Server ------------------------------------------------------------------

#[derive(Clone)]
pub struct Server {
    client: Arc<GrafanaClient>,
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
        let client = GrafanaClient::from_config(&config)?;
        // Viewer mode strips the write tools so Claude only sees read-only ones.
        let mut tool_router = Self::tool_router();
        if connector_core::Mode::from_env("GRAFANA_MODE").is_viewer() {
            tool_router.remove_route("grafana_create_annotation");
        }
        Ok(Self {
            client: Arc::new(client),
            config: Arc::new(config),
            tool_router,
        })
    }

    #[tool(description = "Search Grafana dashboards by title.")]
    async fn grafana_search_dashboards(
        &self,
        Parameters(args): Parameters<SearchDashboardsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50);
        let query = args.query.as_deref().unwrap_or("");
        let text = match self.client.search_dashboards(query, limit).await {
            Ok(d) => format::dashboards(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    fn max_len(&self) -> usize {
        self.config.max_content_length
    }

    #[tool(description = "Get a single Grafana dashboard by its uid, including its panels.")]
    async fn grafana_get_dashboard(
        &self,
        Parameters(args): Parameters<GetDashboardArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.get_dashboard(&args.uid).await {
            Ok(d) => format::dashboard(&d, self.max_len()),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "List configured Grafana data sources.")]
    async fn grafana_list_datasources(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.list_datasources().await {
            Ok(d) => format::datasources(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "List Grafana alert rules (Grafana 9+ unified alerting).")]
    async fn grafana_list_alerts(
        &self,
        Parameters(args): Parameters<ListAlertsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50) as usize;
        let text = match self.client.list_alerts().await {
            Ok(mut d) => {
                if let Some(arr) = d.as_array_mut() {
                    arr.truncate(limit);
                }
                format::alerts(&d)
            }
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Create a Grafana annotation, optionally scoped to a dashboard.")]
    async fn grafana_create_annotation(
        &self,
        Parameters(args): Parameters<CreateAnnotationArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let tags = args.tags.unwrap_or_default();
        let text = match self
            .client
            .create_annotation(&args.text, &tags, args.dashboard_uid.as_deref())
            .await
        {
            Ok(d) => format::created_annotation(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }
}

impl Server {
    /// Make one cheap call to verify the connection works.
    /// Used by the `--test-connection` binary mode.
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        self.client.health().await?;
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
                name: "grafana".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Self-hosted Grafana integration. Use the grafana_* tools to search and read \
                 dashboards, list data sources, and list alert rules. In Writer mode you can \
                 also create annotations. Dashboards are addressed by their uid."
                    .to_string(),
            ),
        }
    }
}
