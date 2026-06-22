//! The `Engine` abstraction shared by every supported database backend.
//!
//! MySQL, MariaDB, ClickHouse and Oracle all expose the same five tools
//! (`list_databases`, `list_tables`, `describe_table`, `run_query`,
//! `server_info`). Each backend implements this trait so the MCP tool surface in
//! [`crate::server`] stays backend-agnostic.

use async_trait::async_trait;

/// One column from a `describe_table` result, normalised across engines.
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    /// Key kind (e.g. "PRI", "MUL", "" -- engine-specific, best effort).
    pub key: String,
}

/// A rendered query result: column headers plus rows of stringified cells.
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// A read-only database backend. Implementations live under `engines/`.
#[async_trait]
pub trait Engine: Send + Sync {
    async fn list_databases(&self) -> anyhow::Result<Vec<String>>;
    async fn list_tables(&self, database: Option<&str>) -> anyhow::Result<Vec<String>>;
    async fn describe_table(&self, table: &str) -> anyhow::Result<Vec<ColumnInfo>>;
    /// Run a read-only query; return column names + string-rendered rows.
    async fn query(&self, sql: &str, limit: u32) -> anyhow::Result<QueryResult>;
    fn name(&self) -> &'static str;
}
