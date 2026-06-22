//! HTTP client for GitHub, built on `connector_core::HttpClient`.
//!
//! API methods are thin wrappers over the GitHub Enterprise Server REST v3
//! endpoints. The `owner` and `repo` path segments are always URL-encoded.

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
    // GitHub rejects requests without these headers (no User-Agent -> 403).
    hc.extra_headers = vec![
        ("Accept".to_string(), "application/vnd.github+json".to_string()),
        ("User-Agent".to_string(), "claude-chat-mcp".to_string()),
    ];
    HttpClient::new(hc)
}

#[derive(Clone)]
pub struct GitHubClient {
    http: HttpClient,
}

impl GitHubClient {
    pub fn from_config(cfg: &Config) -> Result<Self, CoreError> {
        let base = cfg
            .url
            .as_deref()
            .ok_or_else(|| CoreError::Config("GITHUB_URL is not configured.".to_string()))?;
        Ok(Self {
            http: build_client(base, cfg)?,
        })
    }

    pub async fn search_repos(&self, query: &str, limit: u32) -> Result<Value, CoreError> {
        self.http
            .get_json(
                "/search/repositories",
                &[
                    ("q", query.to_string()),
                    ("per_page", limit.to_string()),
                ],
            )
            .await
    }

    pub async fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        state: &str,
        limit: u32,
    ) -> Result<Value, CoreError> {
        let path = format!(
            "/repos/{}/{}/issues",
            urlencoding::encode(owner),
            urlencoding::encode(repo)
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

    pub async fn get_issue(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<Value, CoreError> {
        let path = format!(
            "/repos/{}/{}/issues/{}",
            urlencoding::encode(owner),
            urlencoding::encode(repo),
            number
        );
        self.http.get_json(&path, &[]).await
    }

    pub async fn list_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        state: &str,
        limit: u32,
    ) -> Result<Value, CoreError> {
        let path = format!(
            "/repos/{}/{}/pulls",
            urlencoding::encode(owner),
            urlencoding::encode(repo)
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
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
    ) -> Result<Value, CoreError> {
        let path = format!(
            "/repos/{}/{}/issues",
            urlencoding::encode(owner),
            urlencoding::encode(repo)
        );
        let payload = json!({ "title": title, "body": body });
        self.http
            .send_json(reqwest::Method::POST, &path, payload)
            .await
    }

    pub async fn comment_issue(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
        body_text: &str,
    ) -> Result<Value, CoreError> {
        let path = format!(
            "/repos/{}/{}/issues/{}/comments",
            urlencoding::encode(owner),
            urlencoding::encode(repo),
            number
        );
        let payload = json!({ "body": body_text });
        self.http
            .send_json(reqwest::Method::POST, &path, payload)
            .await
    }

    /// Cheap authenticated call to verify the connection works.
    pub async fn current_user(&self) -> Result<Value, CoreError> {
        self.http.get_json("/user", &[]).await
    }
}
