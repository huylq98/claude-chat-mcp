//! Configuration for the GitHub connector.
//!
//! Self-hosted GitHub Enterprise Server is reached at a single API base URL
//! (e.g. `https://github.corp.com/api/v3`) with a Personal Access Token (sent
//! as a Bearer credential).

use connector_core::env::{env, env_bool, env_u32, env_usize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub url: Option<String>,

    pub token: Option<String>,

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
        let mut proxy_url = env("GITHUB_PROXY_URL");
        connector_core::apply_system_proxy_fallback(&mut proxy_url);
        Self {
            url: clean_url(env("GITHUB_URL")),
            token: env("GITHUB_TOKEN"),
            ssl_verify: env_bool("GITHUB_SSL_VERIFY", true),
            ca_bundle: env("GITHUB_CA_BUNDLE"),
            proxy_url,
            timeout: Duration::from_secs(env_u32("GITHUB_TIMEOUT", 30) as u64),
            rate_limit: env_u32("GITHUB_RATE_LIMIT", 10),
            max_content_length: env_usize("GITHUB_MAX_CONTENT_LENGTH", 50_000),
        }
    }

    /// Require the base URL and a Personal Access Token.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.url.is_none() {
            anyhow::bail!("GITHUB_URL is required (e.g. https://github.corp.com/api/v3).");
        }
        let has_token = self.token.as_deref().map(str::trim).unwrap_or("").len() > 0;
        if !has_token {
            anyhow::bail!(
                "No credentials. Set GITHUB_TOKEN (Personal Access Token with repo or read-only scope)."
            );
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
            token: Some("t".into()),
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
    fn validate_fails_without_token() {
        let mut cfg = base();
        cfg.url = Some("https://github.corp.com/api/v3".into());
        cfg.token = None;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_ok_with_url_and_token() {
        let mut cfg = base();
        cfg.url = Some("https://github.corp.com/api/v3".into());
        assert!(cfg.validate().is_ok());
    }
}
