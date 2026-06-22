//! HTTP client for Elasticsearch / OpenSearch, built on
//! `connector_core::HttpClient`.
//!
//! API methods are thin wrappers over the REST API, which is shared between
//! Elasticsearch and OpenSearch. Index names are URL-encoded into the path.

use crate::config::Config;
use connector_core::{Auth, CoreError, HttpClient, HttpConfig};
use serde_json::Value;

fn build_client(base_url: &str, cfg: &Config) -> Result<HttpClient, CoreError> {
    // Auth is optional: use Basic when a username is set, otherwise None.
    let auth = match cfg.username.as_deref().filter(|u| !u.trim().is_empty()) {
        Some(u) => Auth::Basic {
            username: u.to_string(),
            password: cfg.password.clone().unwrap_or_default(),
        },
        None => Auth::None,
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
pub struct EsClient {
    http: HttpClient,
}

impl EsClient {
    pub fn from_config(cfg: &Config) -> Result<Self, CoreError> {
        let base = cfg
            .url
            .as_deref()
            .ok_or_else(|| CoreError::Config("ES_URL is not configured.".to_string()))?;
        Ok(Self {
            http: build_client(base, cfg)?,
        })
    }

    pub async fn list_indices(&self) -> Result<Value, CoreError> {
        self.http
            .get_json(
                "/_cat/indices",
                &[
                    ("format", "json".to_string()),
                    ("h", "index,health,status,docs.count,store.size".to_string()),
                ],
            )
            .await
    }

    pub async fn get_mapping(&self, index: &str) -> Result<Value, CoreError> {
        let path = format!("/{}/_mapping", urlencoding::encode(index));
        self.http.get_json(&path, &[]).await
    }

    pub async fn search(&self, index: &str, query: &str, size: u32) -> Result<Value, CoreError> {
        let path = format!("/{}/_search", urlencoding::encode(index));
        let mut params = vec![("size", size.to_string())];
        if !query.trim().is_empty() {
            params.push(("q", query.to_string()));
        }
        self.http.get_json(&path, &params).await
    }

    pub async fn index_document(&self, index: &str, document: Value) -> Result<Value, CoreError> {
        let path = format!("/{}/_doc", urlencoding::encode(index));
        self.http
            .send_json(reqwest::Method::POST, &path, document)
            .await
    }

    /// Cheap call to verify the connection works (cluster info at `/`).
    pub async fn cluster_info(&self) -> Result<Value, CoreError> {
        self.http.get_json("/", &[]).await
    }
}
