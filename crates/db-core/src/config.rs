//! Shared connection settings loaded from environment variables.
//!
//! Every binary connector reads the same env vars (`DB_HOST`, `DB_PORT`,
//! `DB_USER`, `DB_PASSWORD`, `DB_NAME`); the engine itself is fixed by the
//! binary, so there is no `DB_ENGINE` field. The caller passes the
//! engine-appropriate default port. No secrets are ever logged or returned by
//! `server_info`.

use connector_core::env::{env, env_u32};

/// Connection settings shared across engines. Each engine reads the fields it
/// needs; the default port is supplied by the caller (engine-specific).
#[derive(Debug, Clone)]
pub struct DbConnConfig {
    pub host: String,
    pub port: u32,
    pub user: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
    /// Oracle service name (`DB_SERVICE`); unused by other engines.
    pub service: Option<String>,
}

impl DbConnConfig {
    /// Build config from env, applying `default_port` when `DB_PORT` is unset.
    pub fn from_env(default_port: u32) -> Self {
        Self {
            host: env("DB_HOST").unwrap_or_else(|| "127.0.0.1".to_string()),
            port: env_u32("DB_PORT", default_port),
            user: env("DB_USER"),
            password: env("DB_PASSWORD"),
            database: env("DB_NAME"),
            service: env("DB_SERVICE"),
        }
    }

    /// Human-readable, secret-free summary for the `server_info` tool.
    pub fn summary(&self, engine_name: &str) -> String {
        let db = self.database.as_deref().unwrap_or("(none)");
        let user = self.user.as_deref().unwrap_or("(none)");
        format!(
            "Engine: {engine_name}\nHost: {}:{}\nUser: {user}\nDatabase/Schema: {db}",
            self.host, self.port
        )
    }
}
