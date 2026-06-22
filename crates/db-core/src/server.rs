//! Generic MCP server: an rmcp handler exposing the five read-only database
//! tools over a single, backend-agnostic surface. Each binary connector builds
//! one of these via [`make_server`], passing in an already-connected engine and
//! the engine's display label; the tool surface itself is identical everywhere.

use crate::config::DbConnConfig;
use crate::engine::Engine;
use crate::sql::{format_columns, format_list, format_rows, guard_read_only};
use rmcp::schemars::JsonSchema;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use std::sync::Arc;

/// Default row cap when the caller omits `limit`.
const DEFAULT_LIMIT: u32 = 100;
/// Hard upper bound on rows returned, regardless of the requested `limit`.
const MAX_LIMIT: u32 = 1000;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListTablesArgs {
    /// Optional database/schema to list tables from. Defaults to the connection's database.
    pub database: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DescribeTableArgs {
    /// The table name to describe.
    pub table: String,
    /// Optional database/schema the table lives in (informational; engine-dependent).
    pub database: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryArgs {
    /// A read-only SQL query (SELECT/SHOW/DESCRIBE/EXPLAIN only).
    pub sql: String,
    /// Max rows to return (default 100, capped at 1000).
    pub limit: Option<u32>,
}

/// A backend-agnostic MCP server over an [`Engine`]. Built by [`make_server`].
#[derive(Clone)]
pub struct DbServer {
    engine: Arc<dyn Engine>,
    config: Arc<DbConnConfig>,
    /// Server name reported in MCP `get_info` (e.g. "mysql").
    label: &'static str,
    tool_router: ToolRouter<DbServer>,
}

/// Build a server handler from a connected engine and a static label.
///
/// `label` is the connector's binary/server name (e.g. "mysql", "clickhouse").
/// `config` is used only for the secret-free `server_info` summary.
pub fn make_server(
    engine: Arc<dyn Engine>,
    config: DbConnConfig,
    label: &'static str,
) -> DbServer {
    DbServer {
        engine,
        config: Arc::new(config),
        label,
        tool_router: DbServer::tool_router(),
    }
}

#[tool_router]
impl DbServer {
    #[tool(description = "List all databases/schemas the connection can see.")]
    async fn list_databases(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.engine.list_databases().await {
            Ok(dbs) => format_list("Databases", &dbs),
            Err(e) => format!("Error: {e}"),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "List tables in a database. If no database is given, uses the connection's default.")]
    async fn list_tables(
        &self,
        Parameters(args): Parameters<ListTablesArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.engine.list_tables(args.database.as_deref()).await {
            Ok(tables) => format_list("Tables", &tables),
            Err(e) => format!("Error: {e}"),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Describe a table's columns (name, type, nullability, key) as a markdown table.")]
    async fn describe_table(
        &self,
        Parameters(args): Parameters<DescribeTableArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.engine.describe_table(&args.table).await {
            Ok(cols) => format!("Columns of `{}`:\n\n{}", args.table, format_columns(&cols)),
            Err(e) => format!("Error: {e}"),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Run a read-only SQL query and return rows as a markdown table.")]
    async fn run_query(
        &self,
        Parameters(args): Parameters<QueryArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
        let text = match guard_read_only(&args.sql) {
            Err(msg) => msg,
            Ok(()) => match self.engine.query(&args.sql, limit).await {
                Ok(result) => format_rows(&result),
                Err(e) => format!("Error: {e}"),
            },
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Show the active engine and connection summary (no secrets).")]
    async fn server_info(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = self.config.summary(self.engine.name());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

#[tool_handler]
impl ServerHandler for DbServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: self.label.into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Read-only database access. Use list_databases / list_tables / \
                 describe_table to explore, then run_query for read-only \
                 SELECT/SHOW/DESCRIBE/EXPLAIN statements."
                    .into(),
            ),
        }
    }
}
