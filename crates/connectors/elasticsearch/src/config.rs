//! Configuration for the Elasticsearch / OpenSearch connector.
//!
//! The cluster is reached at a single base URL. Authentication is optional:
//! some self-hosted clusters are unauthenticated, others use HTTP Basic auth
//! with a username and password. Only the base URL is required.

use connector_core::env::{env, env_bool, env_u32, env_usize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub url: Option<String>,

    pub username: Option<String>,
    pub password: Option<String>,

    pub ssl_verify: bool,
    pub ca_bundle: Option<String>,
    pub proxy_url: Option<String>,
    pub timeout: Duration,
    pub rate_limit: u32,
    pub max_content_length: usize,
}

fn clean_url(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
}

impl Config {
    pub fn from_env() -> Self {
        let mut proxy_url = env("ES_PROXY_URL");
        connector_core::apply_system_proxy_fallback(&mut proxy_url);
        Self {
            url: clean_url(env("ES_URL")),
            username: env("ES_USER"),
            password: env("ES_PASSWORD"),
            ssl_verify: env_bool("ES_SSL_VERIFY", true),
            ca_bundle: env("ES_CA_BUNDLE"),
            proxy_url,
            timeout: Duration::from_secs(env_u32("ES_TIMEOUT", 30) as u64),
            rate_limit: env_u32("ES_RATE_LIMIT", 10),
            max_content_length: env_usize("ES_MAX_CONTENT_LENGTH", 50_000),
        }
    }

    /// Require only the base URL. Credentials are optional because some
    /// clusters are unauthenticated.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.url.is_none() {
            anyhow::bail!("ES_URL is required (e.g. https://es.corp.com:9200).");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Config {
        Config {
            url: None,
            username: None,
            password: None,
            ssl_verify: true,
            ca_bundle: None,
            proxy_url: None,
            timeout: Duration::from_secs(30),
            rate_limit: 10,
            max_content_length: 50_000,
        }
    }

    #[test]
    fn validate_fails_without_url() {
        let cfg = base();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_ok_with_url_only() {
        let mut cfg = base();
        cfg.url = Some("https://es.corp.com:9200".into());
        assert!(cfg.validate().is_ok());
    }
}
