//! PostgreSQL engine over `tokio-postgres` (pure Rust, no system libraries).
//!
//! A Postgres connection is bound to a single database, so `list_tables`
//! enumerates the `public` schema and the `database` argument is ignored.
//! Every cell is stringified via `simple_query`, which returns all columns as
//! text regardless of their wire type, keeping the tool layer uniform.
//!
//! Connections use `NoTls`; TLS / `sslmode` support is a follow-up.

use crate::config::DbConnConfig;
use crate::engine::{ColumnInfo, Engine, QueryResult};
use async_trait::async_trait;
use tokio_postgres::{Client, NoTls, SimpleQueryMessage};

pub struct PostgresEngine {
    client: Client,
}

impl PostgresEngine {
    /// Connect using the shared connection config. The connection future is
    /// spawned onto the Tokio runtime; we keep the [`Client`] for queries.
    pub async fn connect(config: &DbConnConfig) -> anyhow::Result<Self> {
        let mut pg_config = tokio_postgres::Config::new();
        pg_config.host(&config.host);
        pg_config.port(config.port as u16);
        if let Some(user) = &config.user {
            pg_config.user(user);
        }
        if let Some(password) = &config.password {
            pg_config.password(password);
        }
        if let Some(db) = &config.database {
            pg_config.dbname(db);
        }

        let (client, connection) = pg_config.connect(NoTls).await?;

        // The connection performs the actual I/O and must be driven to make
        // progress; spawn it so it runs in the background for the client's life.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("postgres connection error: {e}");
            }
        });

        Ok(Self { client })
    }
}

#[async_trait]
impl Engine for PostgresEngine {
    async fn list_databases(&self) -> anyhow::Result<Vec<String>> {
        let rows = self
            .client
            .query(
                "SELECT datname FROM pg_database WHERE datistemplate = false ORDER BY datname",
                &[],
            )
            .await?;
        Ok(rows.iter().map(|r| r.get::<_, String>(0)).collect())
    }

    async fn list_tables(&self, _database: Option<&str>) -> anyhow::Result<Vec<String>> {
        // Postgres connects to one database; list tables in the public schema.
        let rows = self
            .client
            .query(
                "SELECT tablename FROM pg_tables WHERE schemaname = $1 ORDER BY tablename",
                &[&"public"],
            )
            .await?;
        Ok(rows.iter().map(|r| r.get::<_, String>(0)).collect())
    }

    async fn describe_table(&self, table: &str) -> anyhow::Result<Vec<ColumnInfo>> {
        let rows = self
            .client
            .query(
                "SELECT column_name, data_type, is_nullable \
                 FROM information_schema.columns \
                 WHERE table_name = $1 ORDER BY ordinal_position",
                &[&table],
            )
            .await?;
        // These information_schema columns are all text, so String reads work.
        let columns = rows
            .iter()
            .map(|r| {
                let is_nullable = r.get::<_, String>(2);
                ColumnInfo {
                    name: r.get::<_, String>(0),
                    data_type: r.get::<_, String>(1),
                    nullable: is_nullable == "YES",
                    key: String::new(),
                }
            })
            .collect();
        Ok(columns)
    }

    async fn query(&self, sql: &str, limit: u32) -> anyhow::Result<QueryResult> {
        // `simple_query` returns every column as text regardless of type, which
        // avoids decoding each Postgres type and keeps the output uniform.
        let messages = self.client.simple_query(sql).await?;

        let mut columns: Vec<String> = Vec::new();
        let mut rows: Vec<Vec<String>> = Vec::new();

        for message in messages {
            if let SimpleQueryMessage::Row(row) = message {
                if columns.is_empty() {
                    columns = row.columns().iter().map(|c| c.name().to_string()).collect();
                }
                if rows.len() >= limit as usize {
                    continue;
                }
                let mut cells = Vec::with_capacity(row.columns().len());
                for i in 0..row.columns().len() {
                    cells.push(row.get(i).unwrap_or("NULL").to_string());
                }
                rows.push(cells);
            }
        }

        Ok(QueryResult { columns, rows })
    }

    fn name(&self) -> &'static str {
        "PostgreSQL"
    }
}
