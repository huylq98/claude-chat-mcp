//! HTTP client for Jira, built on `connector_core::HttpClient`.
//!
//! API methods are thin wrappers over the self-hosted (Server / Data Center)
//! REST endpoints under `/rest/api/2/...`.

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
pub struct JiraClient {
    http: HttpClient,
}

impl JiraClient {
    pub fn from_config(cfg: &Config) -> Result<Self, CoreError> {
        let base = cfg
            .url
            .as_deref()
            .ok_or_else(|| CoreError::Config("JIRA_URL is not configured.".to_string()))?;
        Ok(Self {
            http: build_client(base, cfg)?,
        })
    }

    pub async fn search(&self, jql: &str, limit: u32) -> Result<Value, CoreError> {
        self.http
            .get_json(
                "/rest/api/2/search",
                &[
                    ("jql", jql.to_string()),
                    ("maxResults", limit.to_string()),
                    (
                        "fields",
                        "summary,status,assignee,priority,issuetype,updated".to_string(),
                    ),
                ],
            )
            .await
    }

    pub async fn get_issue(&self, key: &str) -> Result<Value, CoreError> {
        self.http
            .get_json(&format!("/rest/api/2/issue/{key}"), &[])
            .await
    }

    pub async fn list_projects(&self) -> Result<Value, CoreError> {
        self.http.get_json("/rest/api/2/project", &[]).await
    }

    pub async fn create_issue(
        &self,
        project_key: &str,
        summary: &str,
        description: &str,
        issue_type: &str,
    ) -> Result<Value, CoreError> {
        let payload = json!({
            "fields": {
                "project": { "key": project_key },
                "summary": summary,
                "description": description,
                "issuetype": { "name": issue_type }
            }
        });
        self.http
            .send_json(reqwest::Method::POST, "/rest/api/2/issue", payload)
            .await
    }

    pub async fn add_comment(&self, issue_key: &str, body: &str) -> Result<Value, CoreError> {
        let path = format!("/rest/api/2/issue/{issue_key}/comment");
        let payload = json!({ "body": body });
        self.http
            .send_json(reqwest::Method::POST, &path, payload)
            .await
    }
}
