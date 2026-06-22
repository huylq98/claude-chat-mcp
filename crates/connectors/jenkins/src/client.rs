//! HTTP client for Jenkins, built on `connector_core::HttpClient`.
//!
//! API methods are thin wrappers over the Jenkins JSON API. Job names are
//! URL-encoded into the `/job/<name>` path segment. Folder jobs (nested
//! `/job/a/job/b`) are out of scope; only top-level job names are handled.

use crate::config::Config;
use connector_core::{Auth, CoreError, HttpClient, HttpConfig};
use serde_json::Value;

fn build_client(base_url: &str, cfg: &Config) -> Result<HttpClient, CoreError> {
    // Jenkins uses HTTP Basic auth: username + API token (token as the password).
    let auth = match (&cfg.username, &cfg.token) {
        (Some(u), Some(t)) => Auth::Basic {
            username: u.clone(),
            password: t.clone(),
        },
        _ => Auth::None,
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
pub struct JenkinsClient {
    http: HttpClient,
}

impl JenkinsClient {
    pub fn from_config(cfg: &Config) -> Result<Self, CoreError> {
        let base = cfg
            .url
            .as_deref()
            .ok_or_else(|| CoreError::Config("JENKINS_URL is not configured.".to_string()))?;
        Ok(Self {
            http: build_client(base, cfg)?,
        })
    }

    pub async fn list_jobs(&self) -> Result<Value, CoreError> {
        self.http
            .get_json(
                "/api/json",
                &[("tree", "jobs[name,url,color]".to_string())],
            )
            .await
    }

    pub async fn get_job(&self, name: &str) -> Result<Value, CoreError> {
        let path = format!("/job/{}/api/json", urlencoding::encode(name));
        self.http.get_json(&path, &[]).await
    }

    pub async fn list_builds(&self, job: &str) -> Result<Value, CoreError> {
        let path = format!("/job/{}/api/json", urlencoding::encode(job));
        self.http
            .get_json(
                &path,
                &[(
                    "tree",
                    "builds[number,result,timestamp,url,duration]".to_string(),
                )],
            )
            .await
    }

    pub async fn get_build(&self, job: &str, number: u64) -> Result<Value, CoreError> {
        let path = format!("/job/{}/{}/api/json", urlencoding::encode(job), number);
        self.http.get_json(&path, &[]).await
    }

    /// Trigger a build. Jenkins returns 201 with no body.
    pub async fn trigger_build(&self, job: &str) -> Result<Value, CoreError> {
        let path = format!("/job/{}/build", urlencoding::encode(job));
        self.http
            .send_empty(reqwest::Method::POST, &path)
            .await
    }

    /// Cheap authenticated call to verify the connection works.
    pub async fn root_info(&self) -> Result<Value, CoreError> {
        self.http
            .get_json("/api/json", &[("tree", "mode".to_string())])
            .await
    }
}
