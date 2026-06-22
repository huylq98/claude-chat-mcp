//! ClickHouse engine over the HTTP interface (port 8123 by default).
//!
//! No native driver: we POST SQL to `/` and request `FORMAT JSONCompact`, which
//! returns `{ "meta": [{"name": ...}], "data": [[...]] }`. Auth is carried by the
//! `X-ClickHouse-User` / `X-ClickHouse-Key` headers configured on the
//! [`HttpClient`].

use crate::config::DbConnConfig;
use crate::engine::{ColumnInfo, Engine, QueryResult};
use async_trait::async_trait;
use connector_core::{Auth, HttpClient, HttpConfig};
use serde_json::Value;

pub struct ClickhouseEngine {
    client: HttpClient,
}

impl ClickhouseEngine {
    pub async fn connect(config: &DbConnConfig) -> anyhow::Result<Self> {
        let base_url = format!("http://{}:{}", config.host, config.port);

        // ClickHouse HTTP auth is conventionally carried via dedicated headers
        // rather than HTTP Basic, so leave `Auth::None` and set headers instead.
        let mut extra_headers = Vec::new();
        let user = config.user.clone().unwrap_or_else(|| "default".to_string());
        extra_headers.push(("X-ClickHouse-User".to_string(), user));
        if let Some(pw) = &config.password {
            extra_headers.push(("X-ClickHouse-Key".to_string(), pw.clone()));
        }
        if let Some(db) = &config.database {
            extra_headers.push(("X-ClickHouse-Database".to_string(), db.clone()));
        }

        let mut http_config = HttpConfig::new(base_url, Auth::None);
        http_config.extra_headers = extra_headers;

        let client = HttpClient::new(http_config)?;
        Ok(Self { client })
    }

    /// Run `sql`, appending `FORMAT JSONCompact`, and parse the response into a
    /// [`QueryResult`].
    async fn run_json_compact(&self, sql: &str) -> anyhow::Result<QueryResult> {
        let body = format!("{}\nFORMAT JSONCompact", sql.trim().trim_end_matches(';'));
        let text = self.client.post_text("/", body).await?;
        parse_json_compact(&text)
    }

    /// Run a query and return only the first column of every row as strings.
    async fn first_column(&self, sql: &str) -> anyhow::Result<Vec<String>> {
        let result = self.run_json_compact(sql).await?;
        Ok(result.rows.into_iter().filter_map(|mut r| {
            if r.is_empty() { None } else { Some(r.remove(0)) }
        }).collect())
    }
}

#[async_trait]
impl Engine for ClickhouseEngine {
    async fn list_databases(&self) -> anyhow::Result<Vec<String>> {
        self.first_column("SHOW DATABASES").await
    }

    async fn list_tables(&self, database: Option<&str>) -> anyhow::Result<Vec<String>> {
        let sql = match database {
            Some(db) => format!("SHOW TABLES FROM `{}`", escape_ident(db)),
            None => "SHOW TABLES".to_string(),
        };
        self.first_column(&sql).await
    }

    async fn describe_table(&self, table: &str) -> anyhow::Result<Vec<ColumnInfo>> {
        // DESCRIBE TABLE returns columns: name, type, default_type,
        // default_expression, comment, codec_expression, ttl_expression.
        let sql = format!("DESCRIBE TABLE `{}`", escape_ident(table));
        let result = self.run_json_compact(&sql).await?;
        let columns = result
            .rows
            .into_iter()
            .map(|row| {
                let data_type = row.get(1).cloned().unwrap_or_default();
                // ClickHouse marks optionality with the Nullable(...) wrapper.
                let nullable = data_type.starts_with("Nullable(");
                ColumnInfo {
                    name: row.into_iter().next().unwrap_or_default(),
                    data_type,
                    nullable,
                    key: String::new(),
                }
            })
            .collect();
        Ok(columns)
    }

    async fn query(&self, sql: &str, limit: u32) -> anyhow::Result<QueryResult> {
        let effective = if sql.to_lowercase().contains(" limit ") {
            sql.to_string()
        } else {
            format!("{} LIMIT {}", sql.trim().trim_end_matches(';').trim_end(), limit)
        };
        let mut result = self.run_json_compact(&effective).await?;
        // Defensive cap in case the server returned more than requested.
        result.rows.truncate(limit as usize);
        Ok(result)
    }

    fn name(&self) -> &'static str {
        "ClickHouse"
    }
}

/// Parse a `FORMAT JSONCompact` payload into a [`QueryResult`]. Every cell is
/// rendered to a display string regardless of its JSON type.
fn parse_json_compact(text: &str) -> anyhow::Result<QueryResult> {
    let json: Value = serde_json::from_str(text)
        .map_err(|e| anyhow::anyhow!("invalid ClickHouse JSON response: {e}; body: {text}"))?;

    let columns: Vec<String> = json
        .get("meta")
        .and_then(Value::as_array)
        .map(|metas| {
            metas
                .iter()
                .map(|m| {
                    m.get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string()
                })
                .collect()
        })
        .unwrap_or_default();

    let rows: Vec<Vec<String>> = json
        .get("data")
        .and_then(Value::as_array)
        .map(|data| {
            data.iter()
                .map(|row| {
                    row.as_array()
                        .map(|cells| cells.iter().map(json_cell_to_string).collect())
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(QueryResult { columns, rows })
}

/// Render a JSON cell to a string: strings unquoted, null -> "NULL", everything
/// else via its compact JSON form.
fn json_cell_to_string(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// Escape backticks in an identifier so it can be safely wrapped in backticks.
fn escape_ident(ident: &str) -> String {
    ident.replace('`', "``")
}
