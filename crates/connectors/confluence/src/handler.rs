//! The Confluence MCP server: one stdio server exposing Confluence tools.

use crate::client::ConfluenceClient;
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
    /// A CQL query, e.g. `type=page AND text~"deployment guide"`.
    pub cql: String,
    /// Max results (1-50, default 10).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPageArgs {
    /// The numeric Confluence page ID.
    pub page_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListLimitArgs {
    /// Max results (default 50).
    pub limit: Option<u32>,
}

// ---- Server ------------------------------------------------------------------

#[derive(Clone)]
pub struct Server {
    client: Arc<ConfluenceClient>,
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
        let client = ConfluenceClient::from_config(&config)?;
        Ok(Self {
            client: Arc::new(client),
            config: Arc::new(config),
            tool_router: Self::tool_router(),
        })
    }

    fn max_len(&self) -> usize {
        self.config.max_content_length
    }

    fn base(&self) -> &str {
        self.config.url.as_deref().unwrap_or("")
    }

    #[tool(description = "Search Confluence (self-hosted) pages using CQL.")]
    async fn confluence_search(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(10).clamp(1, 50);
        let text = match self.client.search(&args.cql, limit).await {
            Ok(d) => format::search(&d, self.base()),
            Err(e) => format::error(&e),
        };
        ok(text)
    }

    #[tool(description = "Retrieve a Confluence page's content by numeric ID.")]
    async fn confluence_get_page(
        &self,
        Parameters(args): Parameters<GetPageArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.get_page(&args.page_id).await {
            Ok(d) => format::page(&d, self.base(), self.max_len()),
            Err(e) => format::error(&e),
        };
        ok(text)
    }

    #[tool(description = "List Confluence spaces the user can access.")]
    async fn confluence_list_spaces(
        &self,
        Parameters(args): Parameters<ListLimitArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(50);
        let text = match self.client.list_spaces(limit).await {
            Ok(d) => format::spaces(&d),
            Err(e) => format::error(&e),
        };
        ok(text)
    }
}

impl Server {
    /// Make one cheap authenticated call to verify the connection works.
    /// Used by the `--test-connection` binary mode.
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        self.client.list_spaces(1).await?;
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
                name: "confluence".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Self-hosted Confluence integration (Data Center / Server). Use the \
                 confluence_* tools to search pages with CQL, read a page by ID, and list \
                 accessible spaces."
                    .to_string(),
            ),
        }
    }
}
