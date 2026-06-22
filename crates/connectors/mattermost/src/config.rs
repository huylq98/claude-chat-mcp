//! Configuration for the Mattermost connector.
//!
//! Self-hosted Mattermost is reached at a single base URL with a Personal
//! Access Token (sent as a Bearer credential).

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
        let mut proxy_url = env("MATTERMOST_PROXY_URL");
        connector_core::apply_system_proxy_fallback(&mut proxy_url);
        Self {
            url: clean_url(env("MATTERMOST_URL")),
            token: env("MATTERMOST_TOKEN"),
            ssl_verify: env_bool("MATTERMOST_SSL_VERIFY", true),
            ca_bundle: env("MATTERMOST_CA_BUNDLE"),
            proxy_url,
            timeout: Duration::from_secs(env_u32("MATTERMOST_TIMEOUT", 30) as u64),
            rate_limit: env_u32("MATTERMOST_RATE_LIMIT", 10),
            max_content_length: env_usize("MATTERMOST_MAX_CONTENT_LENGTH", 50_000),
        }
    }

    /// Require the base URL and a Personal Access Token.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.url.is_none() {
            anyhow::bail!("MATTERMOST_URL is required (e.g. https://chat.corp.com).");
        }
        let has_token = self.token.as_deref().map(str::trim).unwrap_or("").len() > 0;
        if !has_token {
            anyhow::bail!(
                "No credentials. Set MATTERMOST_TOKEN (a personal access token from your account settings)."
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
        cfg.url = Some("https://chat.corp.com".into());
        cfg.token = None;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_ok_with_url_and_token() {
        let mut cfg = base();
        cfg.url = Some("https://chat.corp.com".into());
        assert!(cfg.validate().is_ok());
    }
}
