//! HTTP client for Sentry, built on `connector_core::HttpClient`.
//!
//! API methods are thin wrappers over the self-hosted REST endpoints under
//! `/api/0/...`. Path segments (org slug, project slug, issue id) are always
//! URL-encoded.

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
pub struct SentryClient {
    http: HttpClient,
}

impl SentryClient {
    pub fn from_config(cfg: &Config) -> Result<Self, CoreError> {
        let base = cfg
            .url
            .as_deref()
            .ok_or_else(|| CoreError::Config("SENTRY_URL is not configured.".to_string()))?;
        Ok(Self {
            http: build_client(base, cfg)?,
        })
    }

    pub async fn list_projects(&self) -> Result<Value, CoreError> {
        self.http.get_json("/api/0/projects/", &[]).await
    }

    pub async fn list_issues(
        &self,
        org_slug: &str,
        project_slug: &str,
        query: &str,
        limit: u32,
    ) -> Result<Value, CoreError> {
        let path = format!(
            "/api/0/projects/{}/{}/issues/",
            urlencoding::encode(org_slug),
            urlencoding::encode(project_slug)
        );
        self.http
            .get_json(
                &path,
                &[
                    ("query", query.to_string()),
                    ("limit", limit.to_string()),
                ],
            )
            .await
    }

    pub async fn get_issue(&self, issue_id: &str) -> Result<Value, CoreError> {
        let path = format!("/api/0/issues/{}/", urlencoding::encode(issue_id));
        self.http.get_json(&path, &[]).await
    }

    pub async fn update_issue_status(
        &self,
        issue_id: &str,
        status: &str,
    ) -> Result<Value, CoreError> {
        let path = format!("/api/0/issues/{}/", urlencoding::encode(issue_id));
        let body = json!({ "status": status });
        self.http.send_json(reqwest::Method::PUT, &path, body).await
    }

    /// Cheap authenticated call to verify the connection works.
    pub async fn ping(&self) -> Result<Value, CoreError> {
        self.http.get_json("/api/0/projects/", &[]).await
    }
}
