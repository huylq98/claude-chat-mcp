//! Configuration for the Jira connector.
//!
//! Jira Data Center / Server is reached at a single base URL with a shared
//! credential (a Personal Access Token, or basic auth).

use connector_core::env::{env, env_bool, env_u32, env_usize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub url: Option<String>,

    pub token: Option<String>,
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
        let mut proxy_url = env("JIRA_PROXY_URL");
        connector_core::apply_system_proxy_fallback(&mut proxy_url);
        Self {
            url: clean_url(env("JIRA_URL")),
            token: env("JIRA_TOKEN"),
            username: env("JIRA_USERNAME"),
            password: env("JIRA_PASSWORD"),
            ssl_verify: env_bool("JIRA_SSL_VERIFY", true),
            ca_bundle: env("JIRA_CA_BUNDLE"),
            proxy_url,
            timeout: Duration::from_secs(env_u32("JIRA_TIMEOUT", 30) as u64),
            rate_limit: env_u32("JIRA_RATE_LIMIT", 10),
            max_content_length: env_usize("JIRA_MAX_CONTENT_LENGTH", 50_000),
        }
    }

    /// Require the base URL and some form of credential.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.url.is_none() {
            anyhow::bail!("JIRA_URL is required (e.g. https://jira.corp.com).");
        }
        let has_token = self.token.as_deref().map(str::trim).unwrap_or("").len() > 0;
        let has_basic = self.username.is_some() && self.password.is_some();
        if !has_token && !has_basic {
            anyhow::bail!(
                "No credentials. Set JIRA_TOKEN (Personal Access Token) or both JIRA_USERNAME and JIRA_PASSWORD."
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
    fn validate_ok_with_url_and_token() {
        let mut cfg = base();
        cfg.url = Some("https://jira.corp.com".into());
        assert!(cfg.validate().is_ok());
    }
}
