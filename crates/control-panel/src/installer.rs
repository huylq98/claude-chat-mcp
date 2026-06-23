use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Resolve the download URL for a connector, honoring the `CCMCP_FETCH_BASE`
/// test override (when set, fetch from `<base>/<id><ext>` instead of GitHub).
fn resolve_url(id: &str) -> String {
    if let Some(base) = std::env::var_os("CCMCP_FETCH_BASE") {
        let plat = crate::binaries::current_plat();
        let ext = if cfg!(windows) { ".exe" } else { "" };
        return format!("{}/{id}-{plat}{ext}", base.to_string_lossy());
    }
    crate::binaries::download_url(id)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// On-disk file name for a connector binary.
pub fn binary_file_name(id: &str) -> String {
    #[cfg(windows)]
    {
        format!("{id}.exe")
    }
    #[cfg(not(windows))]
    {
        id.to_string()
    }
}

/// The directory where extracted connector binaries live.
///   Windows: %LOCALAPPDATA%\ClaudeChatMCP\connectors\
///   macOS:   ~/Library/Application Support/ClaudeChatMCP/connectors/
///   Linux:   ~/.local/share/ClaudeChatMCP/connectors/
pub fn default_install_dir() -> PathBuf {
    #[cfg(windows)]
    {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("ClaudeChatMCP").join("connectors")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join("Library")
            .join("Application Support")
            .join("ClaudeChatMCP")
            .join("connectors")
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".local")
            .join("share")
            .join("ClaudeChatMCP")
            .join("connectors")
    }
}

/// Return the connector binary path, downloading + verifying it on first use.
///
/// If a cached binary exists and its sha256 matches the expected hash, it is
/// reused. Otherwise the per-OS binary is downloaded from the app's own
/// `cp-v<version>` release, its sha256 checked against the hash baked into the
/// app (an empty expected hash — the dev placeholder — skips verification),
/// then cached and made executable.
pub fn fetch_connector(id: &str) -> io::Result<PathBuf> {
    let (expected_sha, _expected_size) = crate::binaries::current_os_hash(id).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("no binary manifest entry for connector '{id}'"),
        )
    })?;

    let dir = default_install_dir();
    fs::create_dir_all(&dir)?;
    let target = dir.join(binary_file_name(id));

    // Cache hit: an existing file whose hash matches the expected one.
    if !expected_sha.is_empty() {
        if let Ok(existing) = fs::read(&target) {
            if sha256_hex(&existing) == expected_sha {
                return Ok(target);
            }
        }
    }

    // Download the binary.
    let url = resolve_url(id);
    let resp = reqwest::blocking::Client::builder()
        .build()
        .and_then(|c| c.get(&url).send())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("download failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("download failed: HTTP {} for {url}", resp.status()),
        ));
    }
    let bytes = resp
        .bytes()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("download read failed: {e}")))?;

    // Verify against the baked hash (empty = dev placeholder, skip).
    if !expected_sha.is_empty() {
        let got = sha256_hex(&bytes);
        if got != expected_sha {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("checksum mismatch for '{id}': expected {expected_sha}, got {got}"),
            ));
        }
    }

    write_binary(&target, &bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target, perms)?;
    }
    Ok(target)
}

/// Writes `bytes` to `target`. On Windows a running `.exe` is locked by the
/// loader (ERROR_SHARING_VIOLATION = 32), so if Claude Desktop already launched
/// a previous build we retry while Windows releases the image-section handle.
fn write_binary(target: &Path, bytes: &[u8]) -> io::Result<()> {
    match fs::write(target, bytes) {
        Ok(()) => return Ok(()),
        Err(e) if !is_sharing_violation(&e) => return Err(e),
        Err(_) => {}
    }
    let mut last = io::Error::new(io::ErrorKind::Other, "no retry attempts ran");
    for attempt in 1..=5u64 {
        std::thread::sleep(Duration::from_millis(150 * attempt));
        match fs::write(target, bytes) {
            Ok(()) => return Ok(()),
            Err(e) => last = e,
        }
    }
    Err(last)
}

#[cfg(windows)]
fn is_sharing_violation(e: &io::Error) -> bool {
    e.raw_os_error() == Some(32)
}

#[cfg(not(windows))]
fn is_sharing_violation(_e: &io::Error) -> bool {
    false
}

/// Remove an extracted connector binary (best-effort).
pub fn remove_connector_file(id: &str) {
    let path = default_install_dir().join(binary_file_name(id));
    let _ = fs::remove_file(&path);
}

/// Quick writability probe used before extraction so we can fail with a clear
/// message instead of mid-write.
pub fn probe_writable(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let probe = dir.join(".probe");
    {
        let mut f = fs::File::create(&probe)?;
        f.write_all(b"ok")?;
    }
    fs::remove_file(&probe)
}
