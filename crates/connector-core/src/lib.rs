//! Shared, connector-agnostic building blocks for Claude Chat MCP connectors.
//!
//! A connector crate depends on this for: a JSON-over-HTTP [`HttpClient`] with
//! auth/SSL/proxy/retry baked in, plain-text [`format`] helpers, typed [`env`]
//! readers, and Windows system-proxy / PAC detection.

pub mod env;
pub mod error;
pub mod format;
pub mod http;
pub mod pac;
pub mod system_proxy;

pub use error::CoreError;
pub use format::{strip_html, truncate};
pub use http::{Auth, HttpClient, HttpConfig};

/// If `proxy_url` is unset, inherit the Windows system proxy (the same PAC /
/// static config the user's browser uses). Makes connectors "just work" on
/// corporate networks where a raw client without proxy awareness would fail.
pub fn apply_system_proxy_fallback(proxy_url: &mut Option<String>) {
    let is_empty = proxy_url.as_deref().map(str::trim).unwrap_or("").is_empty();
    if !is_empty {
        return;
    }
    let detected = system_proxy::detect();
    if let Some(p) = detected.display() {
        tracing::info!(proxy = %p, source = "system settings", "auto-applied proxy");
        *proxy_url = Some(p);
    }
}
