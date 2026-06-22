//! MySQL / MariaDB engine.
//!
//! Both share an identical wire protocol, so a single implementation backed by
//! `mysql_async` (pure Rust, no system libraries) covers both. We hold a
//! connection [`Pool`] and stringify every cell so the tool layer can render a
//! uniform markdown table.

use crate::config::DbConnConfig;
use crate::engine::{ColumnInfo, Engine, QueryResult};
use async_trait::async_trait;
use mysql_async::prelude::Queryable;
use mysql_async::{OptsBuilder, Pool, Row, Value as MyValue};

pub struct MysqlEngine {
    pool: Pool,
}

impl MysqlEngine {
    /// Connect (lazily, via a pool) using the shared connection config.
    pub async fn connect(config: &DbConnConfig) -> anyhow::Result<Self> {
        let mut opts = OptsBuilder::default()
            .ip_or_hostname(config.host.clone())
            .tcp_port(config.port as u16);

        if let Some(user) = &config.user {
            opts = opts.user(Some(user.clone()));
        }
        if let Some(password) = &config.password {
            opts = opts.pass(Some(password.clone()));
        }
        if let Some(db) = &config.database {
            opts = opts.db_name(Some(db.clone()));
        }

        let pool = Pool::new(opts);
        Ok(Self { pool })
    }

    /// Fetch a single text column from a `SHOW`-style statement.
    async fn fetch_first_column(&self, sql: &str) -> anyhow::Result<Vec<String>> {
        let mut conn = self.pool.get_conn().await?;
        let rows: Vec<Row> = conn.query(sql).await?;
        Ok(rows
            .iter()
            .map(|r| stringify_value(r.as_ref(0).unwrap_or(&MyValue::NULL)))
            .collect())
    }
}

#[async_trait]
impl Engine for MysqlEngine {
    async fn list_databases(&self) -> anyhow::Result<Vec<String>> {
        self.fetch_first_column("SHOW DATABASES").await
    }

    async fn list_tables(&self, database: Option<&str>) -> anyhow::Result<Vec<String>> {
        let sql = match database {
            Some(db) => format!("SHOW TABLES FROM `{}`", escape_ident(db)),
            None => "SHOW TABLES".to_string(),
        };
        self.fetch_first_column(&sql).await
    }

    async fn describe_table(&self, table: &str) -> anyhow::Result<Vec<ColumnInfo>> {
        let sql = format!("SHOW COLUMNS FROM `{}`", escape_ident(table));
        let mut conn = self.pool.get_conn().await?;
        let rows: Vec<Row> = conn.query(sql).await?;
        // SHOW COLUMNS yields: Field, Type, Null, Key, Default, Extra.
        let columns = rows
            .iter()
            .map(|r| {
                let null = cell(r, "Null", 2);
                ColumnInfo {
                    name: cell(r, "Field", 0),
                    data_type: cell(r, "Type", 1),
                    nullable: null.eq_ignore_ascii_case("YES"),
                    key: cell(r, "Key", 3),
                }
            })
            .collect();
        Ok(columns)
    }

    async fn query(&self, sql: &str, limit: u32) -> anyhow::Result<QueryResult> {
        // Append a LIMIT only when the query doesn't already constrain rows.
        let effective = if sql.to_lowercase().contains(" limit ") {
            sql.to_string()
        } else {
            format!("{} LIMIT {}", sql.trim_end_matches(';').trim_end(), limit)
        };

        let mut conn = self.pool.get_conn().await?;
        let rows: Vec<Row> = conn.query(&effective).await?;

        // Derive column headers from the first row's metadata; fall back to an
        // empty header set when no rows came back.
        let columns: Vec<String> = match rows.first() {
            Some(first) => first
                .columns_ref()
                .iter()
                .map(|c| c.name_str().to_string())
                .collect(),
            None => Vec::new(),
        };

        let mut out_rows: Vec<Vec<String>> = Vec::with_capacity(rows.len().min(limit as usize));
        for row in rows.iter().take(limit as usize) {
            let n = row.columns_ref().len();
            let mut cells = Vec::with_capacity(n);
            for i in 0..n {
                cells.push(stringify_value(row.as_ref(i).unwrap_or(&MyValue::NULL)));
            }
            out_rows.push(cells);
        }

        Ok(QueryResult { columns, rows: out_rows })
    }

    fn name(&self) -> &'static str {
        "MySQL/MariaDB"
    }
}

/// Read a named column from a row, falling back to a positional index when the
/// label isn't present.
fn cell(row: &Row, name: &str, index: usize) -> String {
    if let Some(v) = row.get_opt::<MyValue, _>(name).and_then(Result::ok) {
        return stringify_value(&v);
    }
    stringify_value(row.as_ref(index).unwrap_or(&MyValue::NULL))
}

/// Render a `mysql_async::Value` to a display string. NULL becomes "NULL";
/// bytes are decoded as UTF-8 (lossy) since most text/blob columns are textual.
fn stringify_value(v: &MyValue) -> String {
    match v {
        MyValue::NULL => "NULL".to_string(),
        MyValue::Bytes(b) => String::from_utf8_lossy(b).into_owned(),
        MyValue::Int(i) => i.to_string(),
        MyValue::UInt(u) => u.to_string(),
        MyValue::Float(f) => f.to_string(),
        MyValue::Double(d) => d.to_string(),
        MyValue::Date(y, mo, d, h, mi, s, us) => {
            if *h == 0 && *mi == 0 && *s == 0 && *us == 0 {
                format!("{y:04}-{mo:02}-{d:02}")
            } else if *us == 0 {
                format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}")
            } else {
                format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}.{us:06}")
            }
        }
        MyValue::Time(neg, days, h, mi, s, us) => {
            let sign = if *neg { "-" } else { "" };
            let total_h = (*days as u32) * 24 + *h as u32;
            if *us == 0 {
                format!("{sign}{total_h:02}:{mi:02}:{s:02}")
            } else {
                format!("{sign}{total_h:02}:{mi:02}:{s:02}.{us:06}")
            }
        }
    }
}

/// Escape backticks in an identifier so it can be safely wrapped in backticks.
fn escape_ident(ident: &str) -> String {
    ident.replace('`', "``")
}
