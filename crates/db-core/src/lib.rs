//! Shared logic for the read-only database MCP connectors.
//!
//! This library holds everything common to the MySQL, MariaDB, ClickHouse and
//! Oracle connector binaries: the [`Engine`] trait and its per-backend
//! implementations, the read-only SQL guard, the markdown formatters, the shared
//! [`DbConnConfig`], and a generic [`make_server`] builder that turns an
//! `Arc<dyn Engine>` into a ready-to-serve rmcp [`DbServer`]. The binaries stay
//! tiny: pick a port, connect an engine, call `make_server`, serve.

pub mod config;
pub mod engine;
pub mod engines;
pub mod server;
pub mod sql;

pub use config::DbConnConfig;
pub use engine::{ColumnInfo, Engine, QueryResult};
pub use engines::clickhouse::ClickhouseEngine;
pub use engines::mysql::MysqlEngine;
pub use engines::postgres::PostgresEngine;
#[cfg(feature = "oracle")]
pub use engines::oracle::OracleEngine;
pub use server::{make_server, DbServer};
pub use sql::{format_columns, format_list, format_rows, guard_read_only};
