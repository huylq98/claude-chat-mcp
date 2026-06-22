//! HTTP client for Mattermost, built on `connector_core::HttpClient`.
//!
//! API methods are thin wrappers over the self-hosted REST endpoints under
//! `/api/v4/...`. Team and channel ids are always URL-encoded.

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
pub struct MattermostClient {
    http: HttpClient,
}

impl MattermostClient {
    pub fn from_config(cfg: &Config) -> Result<Self, CoreError> {
        let base = cfg
            .url
            .as_deref()
            .ok_or_else(|| CoreError::Config("MATTERMOST_URL is not configured.".to_string()))?;
        Ok(Self {
            http: build_client(base, cfg)?,
        })
    }

    pub async fn list_teams(&self) -> Result<Value, CoreError> {
        self.http.get_json("/api/v4/users/me/teams", &[]).await
    }

    pub async fn list_channels(&self, team_id: &str) -> Result<Value, CoreError> {
        let path = format!(
            "/api/v4/users/me/teams/{}/channels",
            urlencoding::encode(team_id)
        );
        self.http.get_json(&path, &[]).await
    }

    pub async fn get_posts(&self, channel_id: &str, limit: u32) -> Result<Value, CoreError> {
        let path = format!(
            "/api/v4/channels/{}/posts",
            urlencoding::encode(channel_id)
        );
        self.http
            .get_json(&path, &[("per_page", limit.to_string())])
            .await
    }

    pub async fn create_post(&self, channel_id: &str, message: &str) -> Result<Value, CoreError> {
        let body = json!({ "channel_id": channel_id, "message": message });
        self.http
            .send_json(reqwest::Method::POST, "/api/v4/posts", body)
            .await
    }

    /// Cheap authenticated call to verify the connection works.
    pub async fn current_user(&self) -> Result<Value, CoreError> {
        self.http.get_json("/api/v4/users/me", &[]).await
    }
}
