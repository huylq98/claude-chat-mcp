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
    #[serde(rename = "mac-arm")]
    mac_arm: PerOs,
    #[serde(rename = "mac-intel")]
    mac_intel: PerOs,
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

/// The platform-arch key for the running target: `win` (x64), `mac-arm`
/// (Apple Silicon), `mac-intel` (x86_64 mac), or `linux` (x64). This is the
/// suffix used in both the manifest keys and the release asset names.
pub fn current_plat() -> &'static str {
    if cfg!(windows) {
        "win"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "mac-arm"
        } else {
            "mac-intel"
        }
    } else {
        "linux"
    }
}

/// True if the id has a manifest entry.
pub fn known(id: &str) -> bool {
    doc().binaries.contains_key(id)
}

/// `(sha256, size)` for the running platform, or `None` for an unknown id. An
/// empty sha256 means "skip verification" (dev placeholder).
pub fn current_os_hash(id: &str) -> Option<(&'static str, u64)> {
    let e = doc().binaries.get(id)?;
    let per = match current_plat() {
        "win" => &e.win,
        "mac-arm" => &e.mac_arm,
        "mac-intel" => &e.mac_intel,
        _ => &e.linux,
    };
    Some((per.sha256.as_str(), per.size))
}

/// The release version connector binaries are fetched from (the `cp-v<version>`
/// tag). Sourced from the manifest so it always matches what CI published.
pub fn release_version() -> &'static str {
    doc().version.as_str()
}

/// Release download URL for the connector binary on the running platform. Assets
/// are named `<id>-<plat>` (with `.exe` on Windows) so all platforms coexist in
/// one release.
pub fn download_url(id: &str) -> String {
    let plat = current_plat();
    let ext = if cfg!(windows) { ".exe" } else { "" };
    format!(
        "https://github.com/huylq98/claude-chat-mcp/releases/download/cp-v{ver}/{id}-{plat}{ext}",
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
        let plat = current_plat();
        #[cfg(windows)]
        assert!(url.ends_with(&format!("confluence-{plat}.exe")));
        #[cfg(not(windows))]
        assert!(url.ends_with(&format!("confluence-{plat}")));
    }
}
