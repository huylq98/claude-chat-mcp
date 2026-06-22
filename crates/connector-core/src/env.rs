//! Small helpers for reading typed values from environment variables.
//! Connectors load their `Config` from env vars written by the configurator.

pub fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

pub fn env_bool(key: &str, default: bool) -> bool {
    env(key)
        .map(|v| !matches!(v.to_lowercase().as_str(), "false" | "0" | "no"))
        .unwrap_or(default)
}

pub fn env_u32(key: &str, default: u32) -> u32 {
    env(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub fn env_usize(key: &str, default: usize) -> usize {
    env(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}
