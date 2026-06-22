//! Generic JSON-over-HTTP client shared by every HTTP-based connector.
//!
//! Carries the cross-cutting concerns that every corporate connector needs:
//! pluggable auth, SSL/CA handling, proxy + PAC/WPAD resolution, a concurrency
//! cap (rate limit), and retry/backoff on 429/503. Connector crates build their
//! API methods on top of `HttpClient::get_json` / `send_json` / `send_empty`.

use crate::error::CoreError;
use crate::pac::{looks_like_pac_url, PacResolver};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, AUTHORIZATION},
    Method,
};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

/// How the client authenticates each request.
#[derive(Debug, Clone)]
pub enum Auth {
    None,
    /// `Authorization: Bearer <token>` — PATs (Confluence/Jira DC, Airtable, etc.).
    Bearer(String),
    /// `Authorization: Basic base64(user:pass)`.
    Basic { username: String, password: String },
}

/// Everything needed to construct an [`HttpClient`].
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Base URL with no trailing slash (e.g. `https://wiki.corp.com`).
    pub base_url: String,
    pub auth: Auth,
    pub ssl_verify: bool,
    pub ca_bundle: Option<String>,
    pub proxy_url: Option<String>,
    pub timeout: Duration,
    /// Max concurrent in-flight requests.
    pub rate_limit: u32,
    /// Extra static headers applied to every request.
    pub extra_headers: Vec<(String, String)>,
}

impl HttpConfig {
    pub fn new(base_url: impl Into<String>, auth: Auth) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            auth,
            ssl_verify: true,
            ca_bundle: None,
            proxy_url: None,
            timeout: Duration::from_secs(30),
            rate_limit: 10,
            extra_headers: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct HttpClient {
    http: reqwest::Client,
    base_url: String,
    sem: Arc<Semaphore>,
}

impl HttpClient {
    pub fn new(config: HttpConfig) -> Result<Self, CoreError> {
        let mut headers = HeaderMap::new();
        match &config.auth {
            Auth::None => {}
            Auth::Bearer(token) => {
                let v = HeaderValue::from_str(&format!("Bearer {token}"))
                    .map_err(|e| CoreError::Config(format!("invalid token: {e}")))?;
                headers.insert(AUTHORIZATION, v);
            }
            Auth::Basic { username, password } => {
                use base64::{engine::general_purpose::STANDARD, Engine};
                let creds = STANDARD.encode(format!("{username}:{password}"));
                let v = HeaderValue::from_str(&format!("Basic {creds}"))
                    .map_err(|e| CoreError::Config(format!("invalid basic auth: {e}")))?;
                headers.insert(AUTHORIZATION, v);
            }
        }
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        for (name, value) in &config.extra_headers {
            let n = HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| CoreError::Config(format!("invalid header name '{name}': {e}")))?;
            let v = HeaderValue::from_str(value)
                .map_err(|e| CoreError::Config(format!("invalid header value for '{name}': {e}")))?;
            headers.insert(n, v);
        }

        let mut builder = reqwest::Client::builder()
            .default_headers(headers)
            .danger_accept_invalid_certs(!config.ssl_verify)
            .timeout(config.timeout);

        if let Some(bundle_path) = &config.ca_bundle {
            let pem = std::fs::read(bundle_path).map_err(|e| {
                CoreError::Config(format!("cannot read CA bundle at {bundle_path}: {e}"))
            })?;
            let cert = reqwest::Certificate::from_pem(&pem).map_err(|e| {
                CoreError::Config(format!("invalid CA bundle at {bundle_path}: {e}"))
            })?;
            builder = builder.add_root_certificate(cert);
        }

        if let Some(p) = &config.proxy_url {
            let trimmed = p.trim();
            if !trimmed.is_empty() {
                if looks_like_pac_url(trimmed) {
                    let resolver = Arc::new(PacResolver::new(trimmed.to_string()).map_err(|e| {
                        CoreError::Config(format!("PAC setup failed for '{trimmed}': {e}"))
                    })?);
                    let proxy = reqwest::Proxy::custom(move |url| resolver.resolve(url));
                    builder = builder.proxy(proxy);
                } else {
                    let proxy = reqwest::Proxy::all(trimmed).map_err(|e| {
                        CoreError::Config(format!("invalid proxy URL '{trimmed}': {e}"))
                    })?;
                    builder = builder.proxy(proxy);
                }
            }
        }

        let http = builder.build()?;
        Ok(Self {
            http,
            base_url: config.base_url.clone(),
            sem: Arc::new(Semaphore::new(config.rate_limit.max(1) as usize)),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// GET `{base_url}{path}` with query params, retrying 429/503 with backoff.
    pub async fn get_json(&self, path: &str, query: &[(&str, String)]) -> Result<Value, CoreError> {
        let url = format!("{}{}", self.base_url, path);
        let mut attempt = 0u32;
        let max_retries = 3u32;
        loop {
            let _permit = self.sem.acquire().await.unwrap();
            let response = self
                .http
                .request(Method::GET, &url)
                .query(query)
                .send()
                .await?;
            drop(_permit);

            let status = response.status();
            if status.is_success() {
                return Ok(response.json().await?);
            }
            let retryable = matches!(status.as_u16(), 429 | 503);
            if retryable && attempt < max_retries {
                let backoff_ms = 1000u64 * 2u64.pow(attempt);
                tracing::warn!(status = %status, attempt, backoff_ms, "retrying after rate limit / service unavailable");
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                attempt += 1;
                continue;
            }
            return Err(http_error(response).await);
        }
    }

    /// GET returning the raw response body as text (e.g. ClickHouse HTTP).
    pub async fn get_text(&self, path: &str, query: &[(&str, String)]) -> Result<String, CoreError> {
        let url = format!("{}{}", self.base_url, path);
        let _permit = self.sem.acquire().await.unwrap();
        let response = self
            .http
            .request(Method::GET, &url)
            .query(query)
            .send()
            .await?;
        drop(_permit);
        let status = response.status();
        if status.is_success() {
            return Ok(response.text().await?);
        }
        Err(http_error(response).await)
    }

    /// POST a string body to `{base_url}{path}`, returning the text response.
    pub async fn post_text(&self, path: &str, body: String) -> Result<String, CoreError> {
        let url = format!("{}{}", self.base_url, path);
        let _permit = self.sem.acquire().await.unwrap();
        let response = self.http.post(&url).body(body).send().await?;
        drop(_permit);
        let status = response.status();
        if status.is_success() {
            return Ok(response.text().await?);
        }
        Err(http_error(response).await)
    }

    pub async fn send_json(
        &self,
        method: Method,
        path: &str,
        body: Value,
    ) -> Result<Value, CoreError> {
        let url = format!("{}{}", self.base_url, path);
        let _permit = self.sem.acquire().await.unwrap();
        let response = self.http.request(method, &url).json(&body).send().await?;
        drop(_permit);
        parse_write_response(response).await
    }

    pub async fn send_empty(&self, method: Method, path: &str) -> Result<Value, CoreError> {
        let url = format!("{}{}", self.base_url, path);
        let _permit = self.sem.acquire().await.unwrap();
        let response = self.http.request(method, &url).send().await?;
        drop(_permit);
        parse_write_response(response).await
    }
}

async fn http_error(response: reqwest::Response) -> CoreError {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let snippet: String = body.chars().take(300).collect();
    let message = if snippet.trim().is_empty() {
        status.canonical_reason().unwrap_or("").to_string()
    } else {
        snippet
    };
    CoreError::Http {
        status: status.as_u16(),
        message,
    }
}

async fn parse_write_response(response: reqwest::Response) -> Result<Value, CoreError> {
    let status = response.status();
    if status.is_success() {
        if status.as_u16() == 204 {
            return Ok(Value::Null);
        }
        // Some write endpoints return an empty body on success.
        let text = response.text().await?;
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        return Ok(serde_json::from_str(&text)?);
    }
    Err(http_error(response).await)
}
