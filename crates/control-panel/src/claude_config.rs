use serde::Serialize;
use serde_json::{json, Map, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Prefix for every key this app owns in `mcpServers`.
const KEY_PREFIX: &str = "claude-chat-mcp-";

/// One installed connector entry as read back from claude_desktop_config.json.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledEntry {
    pub id: String,
    pub command: String,
    pub env: Map<String, Value>,
}

/// Default config path for the current platform (verbatim from the confluence
/// configurator).
pub fn default_config_path() -> PathBuf {
    #[cfg(windows)]
    {
        let appdata = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        appdata.join("Claude").join("claude_desktop_config.json")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude_desktop_config.json")
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".config")
            .join("Claude")
            .join("claude_desktop_config.json")
    }
}

/// Scan `mcpServers` for keys starting `claude-chat-mcp-` and return each as an
/// `InstalledEntry` (with the prefix stripped from the id).
pub fn read_installed(path: &Path) -> std::io::Result<Vec<InstalledEntry>> {
    let mut out = Vec::new();
    if !path.is_file() {
        return Ok(out);
    }
    let raw = fs::read_to_string(path)?;
    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return Ok(out),
    };
    let servers = match parsed.pointer("/mcpServers").and_then(Value::as_object) {
        Some(s) => s,
        None => return Ok(out),
    };
    for (key, entry) in servers {
        let Some(id) = key.strip_prefix(KEY_PREFIX) else {
            continue;
        };
        let command = entry
            .pointer("/command")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let env = entry
            .pointer("/env")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        out.push(InstalledEntry {
            id: id.to_string(),
            command,
            env,
        });
    }
    Ok(out)
}

/// Merge a connector entry into `mcpServers` under `claude-chat-mcp-<id>` as
/// `{command, args:[], env}`. Preserves all other entries; keeps the atomic
/// write + `.backup` + `.malformed` handling from the confluence configurator.
pub fn write_entry(
    path: &Path,
    id: &str,
    command: &str,
    env: Map<String, Value>,
) -> std::io::Result<()> {
    let mut doc: Value = if path.is_file() {
        match fs::read_to_string(path)
            .ok()
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
        {
            Some(v) => v,
            None => {
                // Malformed - back up and start fresh.
                let ts = now_secs();
                let backup = path.with_extension(format!("json.malformed.{ts}"));
                let _ = fs::copy(path, &backup);
                json!({"mcpServers": {}})
            }
        }
    } else {
        json!({"mcpServers": {}})
    };

    // Back up a good existing config before mutating it.
    if path.is_file() && doc.pointer("/mcpServers").is_some() {
        let _ = fs::copy(path, path.with_extension("json.backup"));
    }

    let server_entry = json!({
        "command": command,
        "args": [],
        "env": env,
    });

    if doc.get("mcpServers").is_none() {
        doc["mcpServers"] = json!({});
    }
    doc["mcpServers"][format!("{KEY_PREFIX}{id}")] = server_entry;

    atomic_write(path, &doc)
}

/// Remove `claude-chat-mcp-<id>` from `mcpServers`. Preserves other entries.
pub fn remove_entry(path: &Path, id: &str) -> std::io::Result<()> {
    if !path.is_file() {
        return Ok(());
    }
    let mut doc: Value = match fs::read_to_string(path)
        .ok()
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
    {
        Some(v) => v,
        None => return Ok(()),
    };
    if let Some(servers) = doc.get_mut("mcpServers").and_then(Value::as_object_mut) {
        servers.remove(&format!("{KEY_PREFIX}{id}"));
    }
    atomic_write(path, &doc)
}

fn atomic_write(path: &Path, doc: &Value) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(doc).unwrap();
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(text.as_bytes())?;
    }
    fs::rename(&tmp, path)
}

fn now_secs() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        .to_string()
}
