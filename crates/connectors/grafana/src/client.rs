//! HTTP client for Grafana, built on `connector_core::HttpClient`.
//!
//! API methods are thin wrappers over the self-hosted REST endpoints under
//! `/api/...`. Dashboards are addressed by their `uid`.

use crate::config::Config;
use connector_core::{Auth, CoreError, HttpClient, HttpConfig};
use serde_json::{json, Value};

fn build_client(base_url: &str, cfg: &Config) -> Result<HttpClient, CoreError> {
    let auth = if let Some(token) = cfg.token.as_deref().filter(|t| !t.trim().is_empty()) {
        Auth::Bearer(token.to_string())
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
pub struct GrafanaClient {
    http: HttpClient,
}

impl GrafanaClient {
    pub fn from_config(cfg: &Config) -> Result<Self, CoreError> {
        let base = cfg
            .url
            .as_deref()
            .ok_or_else(|| CoreError::Config("GRAFANA_URL is not configured.".to_string()))?;
        Ok(Self {
            http: build_client(base, cfg)?,
        })
    }

    pub async fn search_dashboards(&self, query: &str, limit: u32) -> Result<Value, CoreError> {
        self.http
            .get_json(
                "/api/search",
                &[
                    ("query", query.to_string()),
                    ("type", "dash-db".to_string()),
                    ("limit", limit.to_string()),
                ],
            )
            .await
    }

    pub async fn get_dashboard(&self, uid: &str) -> Result<Value, CoreError> {
        let path = format!("/api/dashboards/uid/{}", urlencoding::encode(uid));
        self.http.get_json(&path, &[]).await
    }

    pub async fn list_datasources(&self) -> Result<Value, CoreError> {
        self.http.get_json("/api/datasources", &[]).await
    }

    pub async fn list_alerts(&self) -> Result<Value, CoreError> {
        self.http
            .get_json("/api/v1/provisioning/alert-rules", &[])
            .await
    }

    pub async fn create_annotation(
        &self,
        text: &str,
        tags: &[String],
        dashboard_uid: Option<&str>,
    ) -> Result<Value, CoreError> {
        let mut body = json!({ "text": text, "tags": tags });
        if let Some(uid) = dashboard_uid.filter(|u| !u.trim().is_empty()) {
            body["dashboardUID"] = json!(uid);
        }
        self.http
            .send_json(reqwest::Method::POST, "/api/annotations", body)
            .await
    }

    /// Cheap call to verify the connection works.
    pub async fn health(&self) -> Result<Value, CoreError> {
        self.http.get_json("/api/health", &[]).await
    }
}
