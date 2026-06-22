//! Builds the shared [`connector_core::HttpClient`] for the Airtable REST and
//! Meta APIs (both hosted at `https://api.airtable.com`). All actual HTTP /
//! auth / SSL / proxy / retry logic lives in `connector-core`; this module only
//! translates [`AirtableConfig`] into an [`HttpConfig`].

use crate::config::AirtableConfig;
use connector_core::{Auth, CoreError, HttpClient, HttpConfig};

/// Airtable's single API host (REST + Meta).
pub const AIRTABLE_BASE_URL: &str = "https://api.airtable.com";

pub fn build_client(config: &AirtableConfig) -> Result<HttpClient, CoreError> {
    let mut http = HttpConfig::new(AIRTABLE_BASE_URL, Auth::Bearer(config.token.clone()));
    http.ssl_verify = config.ssl_verify;
    http.ca_bundle = config.ca_bundle.clone();
    http.proxy_url = config.proxy_url.clone();
    http.timeout = config.timeout;
    http.rate_limit = config.rate_limit;
    HttpClient::new(http)
}
