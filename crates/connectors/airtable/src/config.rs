//! Connector configuration, loaded entirely from `AIRTABLE_*` environment
//! variables written by the configurator/wizard.

use connector_core::{apply_system_proxy_fallback, env, CoreError};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AirtableConfig {
    /// Personal Access Token (PAT) used as a Bearer credential.
    pub token: String,
    pub ssl_verify: bool,
    pub ca_bundle: Option<String>,
    pub proxy_url: Option<String>,
    pub timeout: Duration,
    /// Max concurrent in-flight requests. Airtable allows ~5 req/s per base.
    pub rate_limit: u32,
    /// Cap on total characters returned per record-listing response.
    pub max_content_length: usize,
}

impl AirtableConfig {
    pub fn from_env() -> Self {
        let mut proxy_url = env::env("AIRTABLE_PROXY_URL");
        // Inherit the Windows system proxy when one isn't explicitly set, so the
        // connector "just works" on corporate networks.
        apply_system_proxy_fallback(&mut proxy_url);

        Self {
            token: env::env("AIRTABLE_TOKEN").unwrap_or_default(),
            ssl_verify: env::env_bool("AIRTABLE_SSL_VERIFY", true),
            ca_bundle: env::env("AIRTABLE_CA_BUNDLE"),
            proxy_url,
            timeout: Duration::from_secs(env::env_u32("AIRTABLE_TIMEOUT", 30) as u64),
            rate_limit: env::env_u32("AIRTABLE_RATE_LIMIT", 5),
            max_content_length: env::env_usize("AIRTABLE_MAX_CONTENT_LENGTH", 50_000),
        }
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        if self.token.trim().is_empty() {
            return Err(CoreError::Config("AIRTABLE_TOKEN is required".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_requires_token() {
        let cfg = AirtableConfig {
            token: String::new(),
            ssl_verify: true,
            ca_bundle: None,
            proxy_url: None,
            timeout: Duration::from_secs(30),
            rate_limit: 5,
            max_content_length: 50_000,
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_accepts_token() {
        let cfg = AirtableConfig {
            token: "pat_abc123".into(),
            ssl_verify: true,
            ca_bundle: None,
            proxy_url: None,
            timeout: Duration::from_secs(30),
            rate_limit: 5,
            max_content_length: 50_000,
        };
        assert!(cfg.validate().is_ok());
    }
}
