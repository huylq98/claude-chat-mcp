//! The Airtable MCP server: tool definitions and the rmcp `ServerHandler` impl.

use crate::config::AirtableConfig;
use crate::format;
use rmcp::schemars::JsonSchema;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Tool argument types. Doc comments become parameter descriptions in the schema.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListBasesArgs {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListTablesArgs {
    /// The base ID (e.g. 'appXXXXXXXXXXXXXX').
    pub base_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListRecordsArgs {
    /// The base ID (e.g. 'appXXXXXXXXXXXXXX').
    pub base_id: String,
    /// Table name or table ID.
    pub table: String,
    /// Maximum records to return (default 20, capped at 100).
    pub max_records: Option<u32>,
    /// Optional view name or ID to read records through.
    pub view: Option<String>,
    /// Optional Airtable formula to filter records (e.g. "{Status}='Done'").
    pub filter_by_formula: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRecordArgs {
    /// The base ID (e.g. 'appXXXXXXXXXXXXXX').
    pub base_id: String,
    /// Table name or table ID.
    pub table: String,
    /// The record ID (e.g. 'recXXXXXXXXXXXXXX').
    pub record_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateRecordArgs {
    /// The base ID (e.g. 'appXXXXXXXXXXXXXX').
    pub base_id: String,
    /// Table name or table ID.
    pub table: String,
    /// Field name → value map for the new record. Typecast is enabled, so
    /// string values for select/date/number fields are coerced automatically.
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateRecordArgs {
    /// The base ID (e.g. 'appXXXXXXXXXXXXXX').
    pub base_id: String,
    /// Table name or table ID.
    pub table: String,
    /// The record ID to update (e.g. 'recXXXXXXXXXXXXXX').
    pub record_id: String,
    /// Field name → value map of fields to change. Omitted fields are left as-is.
    pub fields: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AirtableServer {
    client: Arc<connector_core::HttpClient>,
    config: Arc<AirtableConfig>,
    tool_router: ToolRouter<AirtableServer>,
}

#[tool_router]
impl AirtableServer {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = AirtableConfig::from_env();
        config.validate()?;
        let client = crate::client::build_client(&config)?;
        // Viewer mode strips the write tools so Claude only sees read-only ones.
        let mut tool_router = Self::tool_router();
        if connector_core::Mode::from_env("AIRTABLE_MODE").is_viewer() {
            for write_tool in ["create_record", "update_record"] {
                tool_router.remove_route(write_tool);
            }
        }
        Ok(Self {
            client: Arc::new(client),
            config: Arc::new(config),
            tool_router,
        })
    }

    pub fn base_url(&self) -> &str {
        self.client.base_url()
    }

    /// Build a `fields` mutation body with typecast enabled.
    fn fields_body(fields: HashMap<String, serde_json::Value>) -> serde_json::Value {
        serde_json::json!({
            "fields": serde_json::Value::Object(fields.into_iter().collect()),
            "typecast": true,
        })
    }

    #[tool(description = "List all Airtable bases accessible with the configured token.")]
    async fn list_bases(
        &self,
        Parameters(_args): Parameters<ListBasesArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.get_json("/v0/meta/bases", &[]).await {
            Ok(v) => format::format_bases(&v),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "List tables in an Airtable base, with each table's primary field and field names/types.")]
    async fn list_tables(
        &self,
        Parameters(args): Parameters<ListTablesArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!("/v0/meta/bases/{}/tables", args.base_id);
        let text = match self.client.get_json(&path, &[]).await {
            Ok(v) => format::format_tables(&v),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "List records in an Airtable table. Supports a view and an Airtable filter formula.")]
    async fn list_records(
        &self,
        Parameters(args): Parameters<ListRecordsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max_records = args.max_records.unwrap_or(20).clamp(1, 100);
        let mut query: Vec<(&str, String)> = vec![("maxRecords", max_records.to_string())];
        if let Some(view) = &args.view {
            query.push(("view", view.clone()));
        }
        if let Some(formula) = &args.filter_by_formula {
            query.push(("filterByFormula", formula.clone()));
        }

        let path = format!("/v0/{}/{}", args.base_id, urlencoding::encode(&args.table));
        let text = match self.client.get_json(&path, &query).await {
            Ok(v) => format::format_records(&v, self.config.max_content_length),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Retrieve a single Airtable record by its ID.")]
    async fn get_record(
        &self,
        Parameters(args): Parameters<GetRecordArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!(
            "/v0/{}/{}/{}",
            args.base_id,
            urlencoding::encode(&args.table),
            args.record_id
        );
        let text = match self.client.get_json(&path, &[]).await {
            Ok(v) => format::format_record(&v),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Create a new record in an Airtable table. Field values are typecast automatically.")]
    async fn create_record(
        &self,
        Parameters(args): Parameters<CreateRecordArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!("/v0/{}/{}", args.base_id, urlencoding::encode(&args.table));
        let body = Self::fields_body(args.fields);
        let text = match self
            .client
            .send_json(reqwest::Method::POST, &path, body)
            .await
        {
            Ok(v) => format::format_mutation("Created", &v),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Update fields on an existing Airtable record. Omitted fields are left unchanged.")]
    async fn update_record(
        &self,
        Parameters(args): Parameters<UpdateRecordArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!(
            "/v0/{}/{}/{}",
            args.base_id,
            urlencoding::encode(&args.table),
            args.record_id
        );
        let body = Self::fields_body(args.fields);
        let text = match self
            .client
            .send_json(reqwest::Method::PATCH, &path, body)
            .await
        {
            Ok(v) => format::format_mutation("Updated", &v),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

#[tool_handler]
impl ServerHandler for AirtableServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "airtable".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Airtable integration. Use these tools to list bases and tables, read \
                records (optionally filtered by a view or Airtable formula), fetch a \
                single record, and create or update records. Base IDs start with 'app' \
                and record IDs start with 'rec'. The 'table' argument accepts either a \
                table name or a table ID."
                    .into(),
            ),
        }
    }
}
