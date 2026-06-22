//! HTTP client for Bitbucket, built on `connector_core::HttpClient`.
//!
//! API methods are thin wrappers over the self-hosted (Server / Data Center)
//! REST endpoints under `/rest/api/1.0/...`.

use crate::config::Config;
use connector_core::{Auth, CoreError, HttpClient, HttpConfig};
use serde_json::{json, Value};

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
pub struct BitbucketClient {
    http: HttpClient,
}

impl BitbucketClient {
    pub fn from_config(cfg: &Config) -> Result<Self, CoreError> {
        let base = cfg
            .url
            .as_deref()
            .ok_or_else(|| CoreError::Config("BITBUCKET_URL is not configured.".to_string()))?;
        Ok(Self {
            http: build_client(base, cfg)?,
        })
    }

    /// Cheapest authenticated GET, used by `--test-connection`: list one project.
    pub async fn list_projects(&self, limit: u32) -> Result<Value, CoreError> {
        self.http
            .get_json("/rest/api/1.0/projects", &[("limit", limit.to_string())])
            .await
    }

    pub async fn list_repos(&self, project_key: &str, limit: u32) -> Result<Value, CoreError> {
        self.http
            .get_json(
                &format!("/rest/api/1.0/projects/{project_key}/repos"),
                &[("limit", limit.to_string())],
            )
            .await
    }

    pub async fn list_pull_requests(
        &self,
        project_key: &str,
        repo_slug: &str,
        state: &str,
        limit: u32,
    ) -> Result<Value, CoreError> {
        self.http
            .get_json(
                &format!("/rest/api/1.0/projects/{project_key}/repos/{repo_slug}/pull-requests"),
                &[("state", state.to_string()), ("limit", limit.to_string())],
            )
            .await
    }

    pub async fn get_commits(
        &self,
        project_key: &str,
        repo_slug: &str,
        limit: u32,
    ) -> Result<Value, CoreError> {
        self.http
            .get_json(
                &format!("/rest/api/1.0/projects/{project_key}/repos/{repo_slug}/commits"),
                &[("limit", limit.to_string())],
            )
            .await
    }

    pub async fn add_pr_comment(
        &self,
        project_key: &str,
        repo_slug: &str,
        pull_request_id: u64,
        text: &str,
    ) -> Result<Value, CoreError> {
        let path = format!(
            "/rest/api/1.0/projects/{project_key}/repos/{repo_slug}/pull-requests/{pull_request_id}/comments"
        );
        let payload = json!({ "text": text });
        self.http
            .send_json(reqwest::Method::POST, &path, payload)
            .await
    }
}
