//! HTTP client for Confluence, built on `connector_core::HttpClient`.
//!
//! API methods are thin wrappers over the self-hosted (Server / Data Center)
//! REST endpoints under `/rest/api/...`.

use crate::config::Config;
use connector_core::{Auth, CoreError, HttpClient, HttpConfig};
use serde_json::Value;

fn build_client(base_url: &str, cfg: &Config) -> Result<HttpClient, CoreError> {
    let auth = if let Some(token) = cfg.token.as_deref().filter(|t| !t.trim().is_empty()) {
        Auth::Bearer(token.to_string())
    } else if let (Some(u), Some(p)) = (&cfg.username, &cfg.password) {
        Auth::Basic {
            username: u.clone(),
            password: p.clone(),
        }
    } else {
        Auth::None
    };

    let mut hc = HttpConfig::new(base_url, auth);
    hc.ssl_verify = cfg.ssl_verify;
    hc.ca_bundle = cfg.ca_bundle.clone();
    hc.proxy_url = cfg.proxy_url.clone();
    hc.timeout = cfg.timeout;
    hc.rate_limit = cfg.rate_limit;
    HttpClient::new(hc)
}

#[derive(Clone)]
pub struct ConfluenceClient {
    http: HttpClient,
}

impl ConfluenceClient {
    pub fn from_config(cfg: &Config) -> Result<Self, CoreError> {
        let base = cfg
            .url
            .as_deref()
            .ok_or_else(|| CoreError::Config("CONFLUENCE_URL is not configured.".to_string()))?;
        Ok(Self {
            http: build_client(base, cfg)?,
        })
    }

    pub async fn search(&self, cql: &str, limit: u32) -> Result<Value, CoreError> {
        self.http
            .get_json(
                "/rest/api/content/search",
                &[
                    ("cql", cql.to_string()),
                    ("limit", limit.to_string()),
                    ("expand", "space,version".to_string()),
                ],
            )
            .await
    }

    pub async fn get_page(&self, page_id: &str) -> Result<Value, CoreError> {
        self.http
            .get_json(
                &format!("/rest/api/content/{page_id}"),
                &[("expand", "body.storage,version,space,ancestors".to_string())],
            )
            .await
    }

    pub async fn list_spaces(&self, limit: u32) -> Result<Value, CoreError> {
        self.http
            .get_json(
                "/rest/api/space",
                &[
                    ("limit", limit.to_string()),
                    ("expand", "description.plain".to_string()),
                ],
            )
            .await
    }
}
