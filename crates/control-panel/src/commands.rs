use crate::claude_config::{
    default_config_path, read_installed, remove_entry, write_entry, InstalledEntry,
};
use crate::installer::{
    default_install_dir, fetch_connector, probe_writable, remove_connector_file,
};
use crate::registry::{self, Connector};
use serde_json::{json, Map, Value};
use std::collections::HashMap;

/// All connectors from the embedded registry.
#[tauri::command]
pub fn list_connectors() -> Vec<Connector> {
    registry::load()
}

/// All currently-installed connector entries from claude_desktop_config.json.
#[tauri::command]
pub fn list_installed() -> Result<Vec<InstalledEntry>, String> {
    read_installed(&default_config_path()).map_err(|e| e.to_string())
}

/// Extract the connector binary, write its `mcpServers` entry, and return the
/// install path.
///
/// `values` is keyed by the connector's env var names (from the registry).
/// `mode` is "viewer" or "writer"; we add `<UPPER(id)>_MODE = mode` to env.
#[tauri::command]
pub fn install_connector(
    id: String,
    values: HashMap<String, String>,
    mode: String,
) -> Result<String, String> {
    let dir = default_install_dir();
    if let Err(e) = probe_writable(&dir) {
        return Err(format!(
            "Cannot write to {}: {e}. Check permissions or your antivirus.",
            dir.display()
        ));
    }

    let server_path = fetch_connector(&id).map_err(|e| {
        if e.raw_os_error() == Some(32) {
            format!(
                "Failed to save the connector binary: the previous server is still running \
                 and has the file locked. Fully quit Claude Desktop (check the system tray) and \
                 try again. ({e})"
            )
        } else {
            format!(
                "Failed to download the connector: {e}. Check your internet connection or proxy, \
                 then try again."
            )
        }
    })?;

    // Build env from the provided values, dropping empties.
    let mut env: Map<String, Value> = Map::new();
    for (k, v) in values {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            env.insert(k, json!(trimmed));
        }
    }
    // Permission role env var, e.g. AIRTABLE_MODE = viewer.
    env.insert(format!("{}_MODE", id.to_uppercase()), json!(mode));

    let command = server_path.to_string_lossy().replace('\\', "/");
    let config_path = default_config_path();
    write_entry(&config_path, &id, &command, env).map_err(|e| {
        format!("Cannot write Claude Desktop config: {e}. Try running as Administrator.")
    })?;

    Ok(command)
}

/// Verify the credentials/URL/proxy before saving: run the connector binary with
/// `--test-connection` and the entered env. Returns Ok with a message on success,
/// Err with the failure reason. Times out after 15s.
#[tauri::command]
pub fn test_connection(
    id: String,
    values: HashMap<String, String>,
    mode: String,
) -> Result<String, String> {
    let server_path =
        fetch_connector(&id).map_err(|e| format!("Could not prepare the connector: {e}"))?;

    let mut cmd = std::process::Command::new(&server_path);
    cmd.arg("--test-connection");
    for (k, v) in &values {
        let t = v.trim();
        if !t.is_empty() {
            cmd.env(k, t);
        }
    }
    cmd.env(format!("{}_MODE", id.to_uppercase()), &mode);
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Could not run the connector: {e}"))?;

    let timeout = std::time::Duration::from_secs(15);
    let start = std::time::Instant::now();
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => {
                let out = child.wait_with_output().map_err(|e| e.to_string())?;
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                return if status.success() {
                    Ok(if stdout.is_empty() { "Connection OK".into() } else { stdout })
                } else {
                    let msg = if !stderr.is_empty() { stderr } else { stdout };
                    Err(if msg.is_empty() { "Connection failed".into() } else { msg })
                };
            }
            None => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Err("Connection test timed out (15s). Check the URL, network, or proxy.".into());
                }
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
        }
    }
}

/// Remove a connector's `mcpServers` entry and delete its extracted binary.
#[tauri::command]
pub fn uninstall_connector(id: String) -> Result<(), String> {
    let config_path = default_config_path();
    remove_entry(&config_path, &id).map_err(|e| format!("Failed to update config: {e}"))?;
    remove_connector_file(&id);
    Ok(())
}
