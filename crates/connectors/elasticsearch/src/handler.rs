//! The Elasticsearch MCP server: one stdio server exposing Elasticsearch /
//! OpenSearch tools.

use crate::client::EsClient;
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
pub struct GetMappingArgs {
    /// The index name to read the mapping for.
    pub index: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchArgs {
    /// The index name to search.
    pub index: String,
    /// Optional Lucene query string (passed via `q`). Empty matches all.
    pub query: Option<String>,
    /// Max results (1-100, default 10).
    pub size: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IndexDocumentArgs {
    /// The index name to index the document into.
    pub index: String,
    /// The document to index, as a JSON object string.
    pub document: String,
}

// ---- Server ------------------------------------------------------------------

#[derive(Clone)]
pub struct Server {
    client: Arc<EsClient>,
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
        let client = EsClient::from_config(&config)?;
        // Viewer mode strips the write tools so Claude only sees read-only ones.
        let mut tool_router = Self::tool_router();
        if connector_core::Mode::from_env("ES_MODE").is_viewer() {
            for write_tool in ["es_index_document"] {
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

    #[tool(description = "List indices in the cluster with health, status, doc count, and size.")]
    async fn es_list_indices(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.list_indices().await {
            Ok(d) => format::indices(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Get the field mapping (top-level fields and types) for an index.")]
    async fn es_get_mapping(
        &self,
        Parameters(args): Parameters<GetMappingArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.get_mapping(&args.index).await {
            Ok(d) => format::mapping(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Search an index with an optional Lucene query string (empty matches all).")]
    async fn es_search(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let size = args.size.unwrap_or(10).clamp(1, 100);
        let query = args.query.as_deref().unwrap_or("");
        let text = match self.client.search(&args.index, query, size).await {
            Ok(d) => format::search_results(&d, self.max_len()),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Index a JSON document into an index. The document argument is a JSON object string.")]
    async fn es_index_document(
        &self,
        Parameters(args): Parameters<IndexDocumentArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let document = match serde_json::from_str(&args.document) {
            Ok(d) => d,
            Err(e) => return ok(format!("Error: document is not valid JSON: {e}")),
        };
        let text = match self.client.index_document(&args.index, document).await {
            Ok(d) => format::indexed_document(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }
}

impl Server {
    /// Make one cheap call to verify the connection works.
    /// Used by the `--test-connection` binary mode.
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        self.client.cluster_info().await?;
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
                name: "elasticsearch".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Self-hosted Elasticsearch or OpenSearch integration. Use the es_* tools to \
                 list indices, read mappings, and search indices with Lucene query strings. \
                 In Writer mode you can also index documents. Both Elasticsearch and \
                 OpenSearch share this REST API."
                    .to_string(),
            ),
        }
    }
}
