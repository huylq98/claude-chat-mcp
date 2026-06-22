//! HTTP client for Redmine, built on `connector_core::HttpClient`.
//!
//! API methods are thin wrappers over the Redmine REST API. Paths use the
//! `.json` suffix to request JSON responses. Auth is via the
//! `X-Redmine-API-Key` header rather than a bearer token.

use crate::config::Config;
use connector_core::{Auth, CoreError, HttpClient, HttpConfig};
use serde_json::{json, Value};

fn build_client(base_url: &str, cfg: &Config) -> Result<HttpClient, CoreError> {
    // Redmine authenticates via a header, not a bearer credential.
    let mut hc = HttpConfig::new(base_url, Auth::None);
    hc.ssl_verify = cfg.ssl_verify;
    hc.ca_bundle = cfg.ca_bundle.clone();
    hc.proxy_url = cfg.proxy_url.clone();
    hc.timeout = cfg.timeout;
    hc.rate_limit = cfg.rate_limit;
    if let Some(token) = cfg.token.as_deref().filter(|t| !t.trim().is_empty()) {
        hc.extra_headers = vec![("X-Redmine-API-Key".to_string(), token.to_string())];
    }
    HttpClient::new(hc)
}

#[derive(Clone)]
pub struct RedmineClient {
    http: HttpClient,
}

impl RedmineClient {
    pub fn from_config(cfg: &Config) -> Result<Self, CoreError> {
        let base = cfg
            .url
            .as_deref()
            .ok_or_else(|| CoreError::Config("REDMINE_URL is not configured.".to_string()))?;
        Ok(Self {
            http: build_client(base, cfg)?,
        })
    }

    pub async fn list_projects(&self, limit: u32) -> Result<Value, CoreError> {
        self.http
            .get_json("/projects.json", &[("limit", limit.to_string())])
            .await
    }

    pub async fn list_issues(
        &self,
        project_id: Option<&str>,
        status: &str,
        limit: u32,
    ) -> Result<Value, CoreError> {
        let mut params = vec![
            ("status_id", status.to_string()),
            ("limit", limit.to_string()),
        ];
        if let Some(pid) = project_id.filter(|p| !p.trim().is_empty()) {
            params.push(("project_id", pid.to_string()));
        }
        self.http.get_json("/issues.json", &params).await
    }

    pub async fn get_issue(&self, issue_id: u64) -> Result<Value, CoreError> {
        let path = format!("/issues/{}.json", issue_id);
        self.http.get_json(&path, &[]).await
    }

    pub async fn create_issue(
        &self,
        project_id: &str,
        subject: &str,
        description: &str,
    ) -> Result<Value, CoreError> {
        let body = json!({
            "issue": {
                "project_id": project_id,
                "subject": subject,
                "description": description
            }
        });
        self.http
            .send_json(reqwest::Method::POST, "/issues.json", body)
            .await
    }

    pub async fn add_note(&self, issue_id: u64, note: &str) -> Result<Value, CoreError> {
        let path = format!("/issues/{}.json", issue_id);
        let body = json!({ "issue": { "notes": note } });
        self.http
            .send_json(reqwest::Method::PUT, &path, body)
            .await
    }

    /// Cheap authenticated call to verify the connection works.
    pub async fn current_user(&self) -> Result<Value, CoreError> {
        self.http.get_json("/users/current.json", &[]).await
    }
}
