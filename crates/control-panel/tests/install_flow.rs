//! L2 integration tests for the "manage MCP servers" core: writing/reading the
//! Claude Desktop config and the fetch-on-demand download path (against a local
//! mock server). These run headlessly on every OS — no GUI, no real network.

use control_panel::claude_config::{read_installed, remove_entry, write_entry};
use control_panel::installer::fetch_connector;
use serde_json::{json, Map};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;

fn unique_temp_dir(tag: &str) -> PathBuf {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut p = std::env::temp_dir();
    p.push(format!("ccmcp-{tag}-{n}"));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// Minimal HTTP server that answers `bodies.len()` sequential GETs, returning
/// `bodies[i]` for request i. Returns the bound base URL and the join handle.
fn serve(bodies: Vec<Vec<u8>>) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        for body in bodies {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(&body).unwrap();
            let _ = stream.flush();
        }
    });
    (base, handle)
}

#[test]
fn write_then_read_then_remove_preserves_siblings() {
    let dir = unique_temp_dir("cfg");
    let cfg = dir.join("claude_desktop_config.json");
    std::fs::write(
        &cfg,
        r#"{"mcpServers":{"other":{"command":"x","args":[],"env":{}}}}"#,
    )
    .unwrap();

    let mut env = Map::new();
    env.insert("CONFLUENCE_URL".into(), json!("https://wiki.example.com"));
    write_entry(&cfg, "confluence", "/path/confluence", env).unwrap();

    let installed = read_installed(&cfg).unwrap();
    assert!(installed.iter().any(|e| e.id == "confluence"));

    // Pre-existing sibling entry is preserved.
    let raw: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
    assert!(raw.pointer("/mcpServers/other").is_some());
    assert_eq!(
        raw.pointer("/mcpServers/claude-chat-mcp-confluence/env/CONFLUENCE_URL")
            .and_then(|v| v.as_str()),
        Some("https://wiki.example.com")
    );

    remove_entry(&cfg, "confluence").unwrap();
    let after = read_installed(&cfg).unwrap();
    assert!(!after.iter().any(|e| e.id == "confluence"));
    let raw2: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
    assert!(raw2.pointer("/mcpServers/other").is_some());
}

#[test]
fn fetch_rejects_bad_checksum_then_accepts_good_one() {
    // Point the install dir at a temp location for this process.
    let dir = unique_temp_dir("fetch");
    #[cfg(windows)]
    std::env::set_var("LOCALAPPDATA", &dir);
    #[cfg(not(windows))]
    std::env::set_var("HOME", &dir);

    // confluence's baked expected hash is sha256("FAKE-BINARY-BYTES").
    let good = b"FAKE-BINARY-BYTES".to_vec();
    let bad = b"WRONG-BYTES".to_vec();
    let (base, handle) = serve(vec![bad, good.clone()]);
    std::env::set_var("CCMCP_FETCH_BASE", &base);

    // 1) Mismatched bytes are rejected (no file is written).
    let err = fetch_connector("confluence").unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("checksum mismatch"));

    // 2) Correct bytes verify and are cached.
    let path = fetch_connector("confluence").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), good);

    handle.join().unwrap();
    std::env::remove_var("CCMCP_FETCH_BASE");
}

#[test]
fn fetch_unknown_connector_errors() {
    let err = fetch_connector("not-a-real-connector").unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}
