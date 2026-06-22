//! Oracle engine -- FEATURE-GATED behind the `oracle` cargo feature.
//!
//! The `oracle` crate links against Oracle Instant Client (not pure Rust) and
//! exposes a blocking API, so every call is wrapped in
//! `tokio::task::spawn_blocking`. The whole module is compiled only when the
//! `oracle` feature is enabled; the binary returns a clear error otherwise.

use crate::config::DbConnConfig;
use crate::engine::{ColumnInfo, Engine, QueryResult};
use async_trait::async_trait;
use oracle::Connection;
use std::sync::Arc;

pub struct OracleEngine {
    /// Connection params, used to open a fresh blocking connection per call.
    /// (The `oracle::Connection` is not `Sync`, so we connect inside each
    /// `spawn_blocking` rather than sharing one handle.)
    connect_string: Arc<String>,
    user: Arc<String>,
    password: Arc<String>,
}

impl OracleEngine {
    pub async fn connect(config: &DbConnConfig) -> anyhow::Result<Self> {
        let service = config
            .service
            .clone()
            .or_else(|| config.database.clone())
            .ok_or_else(|| {
                anyhow::anyhow!("Oracle requires DB_SERVICE (the service name) to be set.")
            })?;
        let user = config
            .user
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Oracle requires DB_USER to be set."))?;
        let password = config.password.clone().unwrap_or_default();
        let connect_string = format!("//{}:{}/{}", config.host, config.port, service);

        // Validate that we can open a connection up-front, on the blocking pool.
        {
            let user = user.clone();
            let password = password.clone();
            let connect_string = connect_string.clone();
            tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                let probe = Connection::connect(&user, &password, &connect_string)
                    .map_err(|e| anyhow::anyhow!("Oracle connection failed: {e}"))?;
                let _ = probe.close();
                Ok(())
            })
            .await??;
        }

        Ok(Self {
            connect_string: Arc::new(connect_string),
            user: Arc::new(user),
            password: Arc::new(password),
        })
    }

    /// Run a blocking closure that needs a connection on the blocking pool.
    async fn with_conn<T, F>(&self, f: F) -> anyhow::Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> anyhow::Result<T> + Send + 'static,
    {
        let connect_string = self.connect_string.clone();
        let user = self.user.clone();
        let password = self.password.clone();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::connect(user.as_str(), password.as_str(), connect_string.as_str())
                .map_err(|e| anyhow::anyhow!("Oracle connection failed: {e}"))?;
            let result = f(&conn);
            let _ = conn.close();
            result
        })
        .await?
    }
}

#[async_trait]
impl Engine for OracleEngine {
    async fn list_databases(&self) -> anyhow::Result<Vec<String>> {
        self.with_conn(|conn| {
            let mut out = Vec::new();
            let rows = conn
                .query("SELECT username FROM all_users ORDER BY username", &[])
                .map_err(|e| anyhow::anyhow!("query failed: {e}"))?;
            for row in rows {
                let row = row.map_err(|e| anyhow::anyhow!("row error: {e}"))?;
                let name: String = row.get(0).map_err(|e| anyhow::anyhow!("column error: {e}"))?;
                out.push(name);
            }
            Ok(out)
        })
        .await
    }

    async fn list_tables(&self, _database: Option<&str>) -> anyhow::Result<Vec<String>> {
        self.with_conn(|conn| {
            let mut out = Vec::new();
            let rows = conn
                .query("SELECT table_name FROM user_tables ORDER BY table_name", &[])
                .map_err(|e| anyhow::anyhow!("query failed: {e}"))?;
            for row in rows {
                let row = row.map_err(|e| anyhow::anyhow!("row error: {e}"))?;
                let name: String = row.get(0).map_err(|e| anyhow::anyhow!("column error: {e}"))?;
                out.push(name);
            }
            Ok(out)
        })
        .await
    }

    async fn describe_table(&self, table: &str) -> anyhow::Result<Vec<ColumnInfo>> {
        // Oracle stores unquoted identifiers in upper-case in the data dictionary.
        let table_upper = table.to_uppercase();
        self.with_conn(move |conn| {
            let mut out = Vec::new();
            let rows = conn
                .query(
                    "SELECT column_name, data_type, nullable \
                     FROM user_tab_columns WHERE table_name = :1 ORDER BY column_id",
                    &[&table_upper],
                )
                .map_err(|e| anyhow::anyhow!("query failed: {e}"))?;
            for row in rows {
                let row = row.map_err(|e| anyhow::anyhow!("row error: {e}"))?;
                let name: String = row.get(0).map_err(|e| anyhow::anyhow!("column error: {e}"))?;
                let data_type: String = row.get(1).map_err(|e| anyhow::anyhow!("column error: {e}"))?;
                let nullable_flag: String =
                    row.get(2).map_err(|e| anyhow::anyhow!("column error: {e}"))?;
                out.push(ColumnInfo {
                    name,
                    data_type,
                    // 'Y' = nullable, 'N' = NOT NULL in user_tab_columns.
                    nullable: nullable_flag.eq_ignore_ascii_case("Y"),
                    key: String::new(),
                });
            }
            Ok(out)
        })
        .await
    }

    async fn query(&self, sql: &str, limit: u32) -> anyhow::Result<QueryResult> {
        // Oracle has no LIMIT; wrap with a ROWNUM bound unless the query already
        // constrains the row count via ROWNUM or FETCH FIRST.
        let lower = sql.to_lowercase();
        let bounded = if lower.contains("rownum") || lower.contains("fetch first") {
            sql.trim().trim_end_matches(';').to_string()
        } else {
            format!(
                "SELECT * FROM ({}) WHERE ROWNUM <= {}",
                sql.trim().trim_end_matches(';'),
                limit
            )
        };

        self.with_conn(move |conn| {
            let rows_iter = conn
                .query(&bounded, &[])
                .map_err(|e| anyhow::anyhow!("query failed: {e}"))?;

            // Column headers come from the statement's column info.
            let columns: Vec<String> = rows_iter
                .column_info()
                .iter()
                .map(|c| c.name().to_string())
                .collect();
            let col_count = columns.len();

            let mut out_rows = Vec::new();
            for row in rows_iter {
                let row = row.map_err(|e| anyhow::anyhow!("row error: {e}"))?;
                let mut cells = Vec::with_capacity(col_count);
                for i in 0..col_count {
                    // Render every value through its SQL string form; NULL -> "NULL".
                    let v: Option<String> =
                        row.get(i).map_err(|e| anyhow::anyhow!("column error: {e}"))?;
                    cells.push(v.unwrap_or_else(|| "NULL".to_string()));
                }
                out_rows.push(cells);
            }

            Ok(QueryResult { columns, rows: out_rows })
        })
        .await
    }

    fn name(&self) -> &'static str {
        "Oracle"
    }
}
