//! The MongoDB MCP server: tool definitions and the rmcp `ServerHandler` impl.

use crate::config::MongoConfig;
use crate::format;
use bson::{doc, Document};
use futures::TryStreamExt;
use rmcp::schemars::JsonSchema;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use std::sync::Arc;

/// Cap on total characters returned per `mongo_find` response.
const MAX_CONTENT_LENGTH: usize = 50_000;

// ---------------------------------------------------------------------------
// Tool argument types. Doc comments become parameter descriptions in the schema.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDatabasesArgs {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListCollectionsArgs {
    /// Database to list collections from. Defaults to the configured database.
    pub database: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindArgs {
    /// Collection name to query.
    pub collection: String,
    /// MongoDB query filter as a JSON object string (default "{}").
    pub filter: Option<String>,
    /// Maximum documents to return (default 20, capped at 100).
    pub limit: Option<u32>,
    /// Database to query. Defaults to the configured database.
    pub database: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CountArgs {
    /// Collection name to count documents in.
    pub collection: String,
    /// MongoDB query filter as a JSON object string (default "{}").
    pub filter: Option<String>,
    /// Database to query. Defaults to the configured database.
    pub database: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InsertArgs {
    /// Collection name to insert into.
    pub collection: String,
    /// The document to insert, as a JSON object string.
    pub document: String,
    /// Database to insert into. Defaults to the configured database.
    pub database: Option<String>,
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct MongoServer {
    client: mongodb::Client,
    config: Arc<MongoConfig>,
    tool_router: ToolRouter<MongoServer>,
}

#[tool_router]
impl MongoServer {
    pub async fn from_env() -> anyhow::Result<Self> {
        let config = MongoConfig::from_env();
        config.validate()?;
        let client = mongodb::Client::with_uri_str(&config.uri).await?;
        // Viewer mode strips the write tool so Claude only sees read-only ones.
        let mut tool_router = Self::tool_router();
        if connector_core::Mode::from_env("MONGODB_MODE").is_viewer() {
            tool_router.remove_route("mongo_insert");
        }
        Ok(Self {
            client,
            config: Arc::new(config),
            tool_router,
        })
    }

    /// Resolve the database to operate on: the explicit argument, else the
    /// configured default. Errors if neither is set.
    fn resolve_db(&self, database: Option<String>) -> Result<mongodb::Database, String> {
        let name = database
            .filter(|d| !d.trim().is_empty())
            .or_else(|| self.config.database.clone());
        match name {
            Some(n) => Ok(self.client.database(&n)),
            None => Err(
                "No database specified and MONGODB_DATABASE is not set. Pass the 'database' argument."
                    .to_string(),
            ),
        }
    }

    /// Parse a JSON string into a BSON document. Empty/None means an empty filter.
    fn parse_document(input: Option<String>) -> Result<Document, String> {
        let raw = input.unwrap_or_default();
        if raw.trim().is_empty() {
            return Ok(Document::new());
        }
        let value: serde_json::Value =
            serde_json::from_str(&raw).map_err(|e| format!("Invalid JSON: {e}"))?;
        bson::to_document(&value).map_err(|e| format!("Not a JSON object: {e}"))
    }

    #[tool(description = "List the names of all databases on the server.")]
    async fn mongo_list_databases(
        &self,
        Parameters(_args): Parameters<ListDatabasesArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.list_database_names().await {
            Ok(names) => format::format_names("databases", &names),
            Err(e) => format!("Error: {e}"),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "List the collection names in a database (defaults to the configured database).")]
    async fn mongo_list_collections(
        &self,
        Parameters(args): Parameters<ListCollectionsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let db = match self.resolve_db(args.database) {
            Ok(db) => db,
            Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
        };
        let text = match db.list_collection_names().await {
            Ok(names) => format::format_names("collections", &names),
            Err(e) => format!("Error: {e}"),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Find documents in a collection matching an optional JSON filter. Returns up to 'limit' documents (default 20, max 100).")]
    async fn mongo_find(
        &self,
        Parameters(args): Parameters<FindArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let db = match self.resolve_db(args.database) {
            Ok(db) => db,
            Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
        };
        let filter = match Self::parse_document(args.filter) {
            Ok(f) => f,
            Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
        };
        let limit = args.limit.unwrap_or(20).clamp(1, 100);

        let collection = db.collection::<Document>(&args.collection);
        let text = match collection.find(filter).limit(limit as i64).await {
            Ok(cursor) => match cursor.try_collect::<Vec<Document>>().await {
                Ok(docs) => format::format_documents(&docs, MAX_CONTENT_LENGTH),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Count documents in a collection matching an optional JSON filter.")]
    async fn mongo_count(
        &self,
        Parameters(args): Parameters<CountArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let db = match self.resolve_db(args.database) {
            Ok(db) => db,
            Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
        };
        let filter = match Self::parse_document(args.filter) {
            Ok(f) => f,
            Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
        };
        let collection = db.collection::<Document>(&args.collection);
        let text = match collection.count_documents(filter).await {
            Ok(n) => format!("{n} document(s) match."),
            Err(e) => format!("Error: {e}"),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Insert a single document (given as a JSON object) into a collection. Returns the inserted id.")]
    async fn mongo_insert(
        &self,
        Parameters(args): Parameters<InsertArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let db = match self.resolve_db(args.database) {
            Ok(db) => db,
            Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
        };
        let document = match Self::parse_document(Some(args.document)) {
            Ok(d) => d,
            Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
        };
        let collection = db.collection::<Document>(&args.collection);
        let text = match collection.insert_one(document).await {
            Ok(result) => format!("Inserted document with id: {}", result.inserted_id),
            Err(e) => format!("Error: {e}"),
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

impl MongoServer {
    /// Run a cheap `ping` to verify the connection works. Pings the configured
    /// default database, falling back to `admin`. Used by `--test-connection`.
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        let db_name = self.config.database.as_deref().unwrap_or("admin");
        self.client
            .database(db_name)
            .run_command(doc! { "ping": 1 })
            .await?;
        Ok(())
    }
}

#[tool_handler]
impl ServerHandler for MongoServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "mongodb".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "MongoDB integration. Use these tools to list databases and \
                collections, find and count documents (filters are JSON object \
                strings, e.g. '{\"status\": \"active\"}'), and insert documents \
                in Writer mode. Collection operations use the configured default \
                database unless a 'database' argument is given."
                    .into(),
            ),
        }
    }
}
