//! HTTP client for GitLab, built on `connector_core::HttpClient`.
//!
//! API methods are thin wrappers over the self-hosted REST endpoints under
//! `/api/v4/...`. Project ids may be a numeric id or a `group/project` path and
//! are always URL-encoded.

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
pub struct GitLabClient {
    http: HttpClient,
}

impl GitLabClient {
    pub fn from_config(cfg: &Config) -> Result<Self, CoreError> {
        let base = cfg
            .url
            .as_deref()
            .ok_or_else(|| CoreError::Config("GITLAB_URL is not configured.".to_string()))?;
        Ok(Self {
            http: build_client(base, cfg)?,
        })
    }

    pub async fn search_projects(&self, search: &str, limit: u32) -> Result<Value, CoreError> {
        self.http
            .get_json(
                "/api/v4/projects",
                &[
                    ("search", search.to_string()),
                    ("membership", "true".to_string()),
                    ("per_page", limit.to_string()),
                ],
            )
            .await
    }

    pub async fn list_issues(
        &self,
        project_id: &str,
        state: &str,
        limit: u32,
    ) -> Result<Value, CoreError> {
        let path = format!("/api/v4/projects/{}/issues", urlencoding::encode(project_id));
        self.http
            .get_json(
                &path,
                &[
                    ("state", state.to_string()),
                    ("per_page", limit.to_string()),
                ],
            )
            .await
    }

    pub async fn get_issue(&self, project_id: &str, issue_iid: u64) -> Result<Value, CoreError> {
        let path = format!(
            "/api/v4/projects/{}/issues/{}",
            urlencoding::encode(project_id),
            issue_iid
        );
        self.http.get_json(&path, &[]).await
    }

    pub async fn list_merge_requests(
        &self,
        project_id: &str,
        state: &str,
        limit: u32,
    ) -> Result<Value, CoreError> {
        let path = format!(
            "/api/v4/projects/{}/merge_requests",
            urlencoding::encode(project_id)
        );
        self.http
            .get_json(
                &path,
                &[
                    ("state", state.to_string()),
                    ("per_page", limit.to_string()),
                ],
            )
            .await
    }

    pub async fn create_issue(
        &self,
        project_id: &str,
        title: &str,
        description: &str,
    ) -> Result<Value, CoreError> {
        let path = format!("/api/v4/projects/{}/issues", urlencoding::encode(project_id));
        let body = json!({ "title": title, "description": description });
        self.http
            .send_json(reqwest::Method::POST, &path, body)
            .await
    }

    pub async fn comment_issue(
        &self,
        project_id: &str,
        issue_iid: u64,
        body_text: &str,
    ) -> Result<Value, CoreError> {
        let path = format!(
            "/api/v4/projects/{}/issues/{}/notes",
            urlencoding::encode(project_id),
            issue_iid
        );
        let body = json!({ "body": body_text });
        self.http
            .send_json(reqwest::Method::POST, &path, body)
            .await
    }

    /// Cheap authenticated call to verify the connection works.
    pub async fn current_user(&self) -> Result<Value, CoreError> {
        self.http.get_json("/api/v4/user", &[]).await
    }
}
