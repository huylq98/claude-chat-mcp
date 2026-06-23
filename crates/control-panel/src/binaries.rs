//! Per-connector binary manifest, embedded at compile time.
//!
//! `resources/binaries.json` lists, for every connector and OS, the sha256 and
//! size of the release binary. The app uses it to (a) know the version/release
//! to download from and (b) verify a downloaded binary before trusting it. CI
//! regenerates this file with real hashes (see `scripts/gen-binaries-json.mjs`);
//! the committed copy is a dev placeholder (empty hashes skip verification).

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

const BINARIES_JSON: &str = include_str!("../resources/binaries.json");

#[derive(Debug, Deserialize)]
struct PerOs {
    sha256: String,
    size: u64,
}

#[derive(Debug, Deserialize)]
struct Entry {
    win: PerOs,
    mac: PerOs,
    linux: PerOs,
}

#[derive(Debug, Deserialize)]
struct Doc {
    version: String,
    binaries: HashMap<String, Entry>,
}

fn doc() -> &'static Doc {
    static D: OnceLock<Doc> = OnceLock::new();
    D.get_or_init(|| {
        serde_json::from_str(BINARIES_JSON).expect("embedded binaries.json is malformed")
    })
}

/// True if the id has a manifest entry.
pub fn known(id: &str) -> bool {
    doc().binaries.contains_key(id)
}

/// `(sha256, size)` for the running OS, or `None` for an unknown id. An empty
/// sha256 means "skip verification" (dev placeholder).
pub fn current_os_hash(id: &str) -> Option<(&'static str, u64)> {
    let e = doc().binaries.get(id)?;
    let per = if cfg!(windows) {
        &e.win
    } else if cfg!(target_os = "macos") {
        &e.mac
    } else {
        &e.linux
    };
    Some((per.sha256.as_str(), per.size))
}

/// The release version connector binaries are fetched from (the `cp-v<version>`
/// tag). Sourced from the manifest so it always matches what CI published.
pub fn release_version() -> &'static str {
    doc().version.as_str()
}

/// Release download URL for the connector binary on the running OS.
pub fn download_url(id: &str) -> String {
    let ext = if cfg!(windows) { ".exe" } else { "" };
    format!(
        "https://github.com/huylq98/claude-chat-mcp/releases/download/cp-v{ver}/{id}{ext}",
        ver = release_version(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_embedded_and_builds_download_url() {
        assert!(known("confluence"));
        assert!(!known("does-not-exist"));

        let (sha, _size) = current_os_hash("confluence").unwrap();
        assert_eq!(sha.len(), 64, "confluence carries a real test hash");
        assert!(current_os_hash("does-not-exist").is_none());

        let url = download_url("confluence");
        assert!(url.contains("/releases/download/cp-v0.14.0/"));
        #[cfg(windows)]
        assert!(url.ends_with("confluence.exe"));
        #[cfg(not(windows))]
        assert!(url.ends_with("confluence"));
    }
}
