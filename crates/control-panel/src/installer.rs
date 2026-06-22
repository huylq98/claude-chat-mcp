use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Returns the embedded release bytes for a connector id, or `None` if the id
/// is unknown.
///
/// Each `crates/control-panel/resources/<id>.exe` (on Windows) or
/// `resources/<id>` (other platforms) must exist at compile time. The lead
/// populates `resources/`.
pub fn embedded_binary(id: &str) -> Option<&'static [u8]> {
    macro_rules! bin {
        ($name:literal) => {{
            #[cfg(windows)]
            {
                Some(include_bytes!(concat!("../resources/", $name, ".exe")) as &'static [u8])
            }
            #[cfg(not(windows))]
            {
                Some(include_bytes!(concat!("../resources/", $name)) as &'static [u8])
            }
        }};
    }

    match id {
        "confluence" => bin!("confluence"),
        "jira" => bin!("jira"),
        "bitbucket" => bin!("bitbucket"),
        "airtable" => bin!("airtable"),
        "mysql" => bin!("mysql"),
        "mariadb" => bin!("mariadb"),
        "clickhouse" => bin!("clickhouse"),
        "oracle" => bin!("oracle"),
        "gitlab" => bin!("gitlab"),
        "postgres" => bin!("postgres"),
        "github" => bin!("github"),
        "jenkins" => bin!("jenkins"),
        "redmine" => bin!("redmine"),
        "grafana" => bin!("grafana"),
        "elasticsearch" => bin!("elasticsearch"),
        _ => None,
    }
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

/// Extract a connector's embedded binary into the install dir and return its
/// path. Errors if the id has no embedded binary or the write fails.
pub fn extract_connector(id: &str) -> io::Result<PathBuf> {
    let bytes = embedded_binary(id).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("no embedded binary for connector '{id}'"),
        )
    })?;
    let dir = default_install_dir();
    fs::create_dir_all(&dir)?;
    let target = dir.join(binary_file_name(id));
    write_binary(&target, bytes)?;
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
