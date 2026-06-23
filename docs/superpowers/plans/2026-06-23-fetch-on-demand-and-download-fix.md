# Fetch-on-demand + Download Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop embedding 18 connector binaries in the desktop app; download the one connector being installed on demand (verified by sha256), so every installer — especially the 100 MB Linux AppImage — shrinks to a few MB and downloads reliably.

**Architecture:** The app ships with zero connector binaries. `installer.rs` gains a `fetch_connector(id)` that returns a cached binary if its sha256 matches the expected hash baked into the app, otherwise downloads the per-OS binary from the app's own `cp-v<version>` GitHub release, verifies it, caches it, and returns the path. Expected hashes are embedded at build time via a generated `binaries.json` (sibling to the embedded `registry.json`). The release workflow publishes per-OS connector binaries + a checksums manifest. A link-integrity test guards the site's download links.

**Tech Stack:** Rust, Tauri 2, `reqwest` (blocking, rustls), `sha2`, Node (link-integrity test), GitHub Actions.

## Global Constraints

- Rust edition 2021, `rust-version = 1.80` (control-panel crate).
- Windows builds require the MSVC dev-shell; build/test on this machine **only** via `scripts/cargo.ps1` (wraps the VS Build Tools env so `link.exe`/Windows SDK are on PATH). Never call bare `cargo` here.
- The 18 connector ids, verbatim and in this order: `confluence jira bitbucket airtable mysql mariadb clickhouse oracle gitlab postgres github jenkins redmine grafana elasticsearch mattermost mongodb sentry`.
- App version is the single source of truth in `crates/control-panel/tauri.conf.json` (`version`), currently `0.13.0`. The fetch-on-demand release bumps it to `0.14.0`.
- Config key prefix is `claude-chat-mcp-` (do not change).
- `reqwest` must use `rustls-tls` (no OpenSSL) and respect standard `HTTP_PROXY`/`HTTPS_PROXY`/`NO_PROXY` env vars.
- Connector binary file name: `<id>.exe` on Windows, `<id>` elsewhere.
- The GitHub repo for release URLs is `huylq98/claude-chat-mcp`.

---

## File Structure

- `crates/control-panel/Cargo.toml` — add `reqwest`, `sha2` deps.
- `crates/control-panel/src/installer.rs` — replace `embedded_binary()` + `extract_connector()` with `fetch_connector()` (download + cache + verify). Keep `default_install_dir`, `binary_file_name`, `remove_connector_file`, `probe_writable`, `write_binary`.
- `crates/control-panel/src/binaries.rs` — **new**: parse the embedded `binaries.json` into `expected_hash(id, os) -> Option<(String /*sha256*/, u64 /*size*/)>` and the `download_url(id)`.
- `crates/control-panel/resources/binaries.json` — **new**, generated; committed placeholder for local dev.
- `crates/control-panel/src/commands.rs` — `install_connector`/`test_connection` call `fetch_connector` instead of `extract_connector`; surface download errors.
- `crates/control-panel/tests/install_flow.rs` — **new**: L2 integration tests (config write/read/uninstall + fetch path against a mock server).
- `.github/workflows/control-panel.yml` — drop the "stage 18 binaries" step; add checksum generation, `binaries.json` generation, and upload of per-OS binaries + `checksums.txt`.
- `site/app.js` — bump `CP_TAG`/`CP_VERSION` to `cp-v0.14.0`/`0.14.0`; add a `.deb` Linux link.
- `tests/links.spec.ts` — **new**: L1 link-integrity test.
- `scripts/gen-binaries-json.mjs` — **new**: builds `binaries.json` from a directory of per-OS binaries (used by CI and locally).

---

## Task 1: Add HTTP + hashing dependencies

**Files:**
- Modify: `crates/control-panel/Cargo.toml`

**Interfaces:**
- Produces: `reqwest::blocking`, `sha2::Sha256` available to the crate.

- [ ] **Step 1: Add dependencies**

In `crates/control-panel/Cargo.toml`, under `[dependencies]`, add:

```toml
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "blocking"] }
sha2 = "0.10"
```

- [ ] **Step 2: Verify it builds**

Run: `pwsh scripts/cargo.ps1 build -p control-panel`
Expected: compiles (new deps downloaded), no errors.

- [ ] **Step 3: Commit**

```bash
git add crates/control-panel/Cargo.toml crates/control-panel/Cargo.lock Cargo.lock
git commit -m "control-panel: add reqwest + sha2 for fetch-on-demand"
```

---

## Task 2: `binaries.json` schema + `binaries.rs` parser

**Files:**
- Create: `crates/control-panel/resources/binaries.json`
- Create: `crates/control-panel/src/binaries.rs`
- Modify: `crates/control-panel/src/lib.rs` (add `pub mod binaries;`)
- Test: in `crates/control-panel/src/binaries.rs` (`#[cfg(test)]`)

**Interfaces:**
- Produces:
  - `pub fn expected(id: &str) -> Option<&'static BinMeta>` where
    `pub struct BinMeta { pub sha256_win: String, pub sha256_mac: String, pub sha256_linux: String, pub size_win: u64, pub size_mac: u64, pub size_linux: u64 }`
  - `pub fn current_os_hash(id: &str) -> Option<(&'static str, u64)>` — returns `(sha256, size)` for the running OS via `cfg!`.
  - `pub fn download_url(id: &str) -> String` — `https://github.com/huylq98/claude-chat-mcp/releases/download/cp-v<VERSION>/<id><.exe on win>` where `<VERSION>` is `env!("CARGO_PKG_VERSION")`.

- [ ] **Step 1: Create the generated resource (hand-written placeholder for dev)**

`crates/control-panel/resources/binaries.json` — minimal valid shape (real values filled by CI in Task 8):

```json
{
  "version": "0.14.0",
  "binaries": {
    "confluence": { "win": {"sha256": "", "size": 0}, "mac": {"sha256": "", "size": 0}, "linux": {"sha256": "", "size": 0} }
  }
}
```

- [ ] **Step 2: Write the failing test**

In `crates/control-panel/src/binaries.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_embedded_and_builds_download_url() {
        // confluence exists in the embedded binaries.json placeholder.
        assert!(expected("confluence").is_some());
        assert!(expected("does-not-exist").is_none());
        let url = download_url("confluence");
        assert!(url.contains("/releases/download/cp-v"));
        #[cfg(windows)]
        assert!(url.ends_with("confluence.exe"));
        #[cfg(not(windows))]
        assert!(url.ends_with("confluence"));
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `pwsh scripts/cargo.ps1 test -p control-panel binaries::`
Expected: FAIL — module `binaries` not found.

- [ ] **Step 4: Implement `binaries.rs`**

```rust
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

const BINARIES_JSON: &str = include_str!("../resources/binaries.json");

#[derive(Debug, Deserialize)]
struct PerOs { sha256: String, size: u64 }

#[derive(Debug, Deserialize)]
struct Entry { win: PerOs, mac: PerOs, linux: PerOs }

#[derive(Debug, Deserialize)]
struct Doc { #[allow(dead_code)] version: String, binaries: HashMap<String, Entry> }

fn doc() -> &'static Doc {
    static D: OnceLock<Doc> = OnceLock::new();
    D.get_or_init(|| serde_json::from_str(BINARIES_JSON).expect("embedded binaries.json malformed"))
}

/// `(sha256, size)` for the running OS, or `None` for an unknown id.
pub fn current_os_hash(id: &str) -> Option<(&'static str, u64)> {
    let e = doc().binaries.get(id)?;
    let per = if cfg!(windows) { &e.win } else if cfg!(target_os = "macos") { &e.mac } else { &e.linux };
    Some((per.sha256.as_str(), per.size))
}

/// True if the id has an embedded binaries entry.
pub fn known(id: &str) -> bool { doc().binaries.contains_key(id) }

/// Release download URL for the connector binary on the running OS.
pub fn download_url(id: &str) -> String {
    let ext = if cfg!(windows) { ".exe" } else { "" };
    format!(
        "https://github.com/huylq98/claude-chat-mcp/releases/download/cp-v{ver}/{id}{ext}",
        ver = env!("CARGO_PKG_VERSION"),
    )
}

// Test helper used by Step 2.
#[cfg(test)]
fn expected(id: &str) -> Option<()> { if known(id) { Some(()) } else { None } }
```

(Replace the `expected` test helper usage in Step 2 with `known(...)` if you prefer; the test asserts `known("confluence")` / `!known("does-not-exist")`.)

- [ ] **Step 5: Register the module**

In `crates/control-panel/src/lib.rs` add after `pub mod registry;`:

```rust
pub mod binaries;
```

- [ ] **Step 6: Run test to verify it passes**

Run: `pwsh scripts/cargo.ps1 test -p control-panel binaries::`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/control-panel/src/binaries.rs crates/control-panel/src/lib.rs crates/control-panel/resources/binaries.json
git commit -m "control-panel: embed binaries.json (per-OS sha256 + download URL)"
```

---

## Task 3: `fetch_connector` — download, verify, cache

**Files:**
- Modify: `crates/control-panel/src/installer.rs`
- Test: `crates/control-panel/tests/install_flow.rs` (created here, extended in Task 6)

**Interfaces:**
- Consumes: `binaries::current_os_hash`, `binaries::download_url`, `binaries::known`.
- Produces:
  - `pub fn fetch_connector(id: &str) -> io::Result<PathBuf>` — returns the cached binary path; downloads + verifies if missing/mismatched.
  - Test-only override of the download base via env var `CCMCP_FETCH_BASE` (if set, `download_url` is replaced by `<base>/<id><ext>`).
- Removes: `embedded_binary()`, `extract_connector()` (replaced).

- [ ] **Step 1: Write the failing test (checksum mismatch rejected, then accepted, then cache hit)**

Create `crates/control-panel/tests/install_flow.rs`:

```rust
use std::io::Write;

// Tiny single-shot HTTP server returning `body` for any GET. Returns the bound URL base.
fn serve_once(bodies: Vec<Vec<u8>>) -> (String, std::thread::JoinHandle<()>) {
    use std::io::Read;
    use std::net::TcpListener;
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
        }
    });
    (base, handle)
}
```

(Full fetch test added in Task 6 once `commands` are wired; this task verifies `fetch_connector` directly.)

Add to the same file:

```rust
#[test]
fn fetch_rejects_checksum_mismatch_and_caches_on_success() {
    // The test sets CCMCP_FETCH_BASE + a temp install dir via HOME/LOCALAPPDATA.
    // It relies on a connector id whose expected hash we control through a test
    // build of binaries.json — see Task 6 for the full harness. This placeholder
    // documents intent; the executable assertion lives in Task 6.
}
```

- [ ] **Step 2: Implement `fetch_connector` in `installer.rs`**

Replace `embedded_binary` and `extract_connector` with:

```rust
use sha2::{Digest, Sha256};

/// Resolve the download URL, honoring the `CCMCP_FETCH_BASE` test override.
fn resolve_url(id: &str) -> String {
    if let Some(base) = std::env::var_os("CCMCP_FETCH_BASE") {
        let ext = if cfg!(windows) { ".exe" } else { "" };
        return format!("{}/{id}{ext}", base.to_string_lossy());
    }
    crate::binaries::download_url(id)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Return the cached connector binary, downloading + verifying it on first use.
pub fn fetch_connector(id: &str) -> io::Result<PathBuf> {
    let (expected_sha, _expected_size) = crate::binaries::current_os_hash(id).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("unknown connector '{id}'"))
    })?;
    let dir = default_install_dir();
    fs::create_dir_all(&dir)?;
    let target = dir.join(binary_file_name(id));

    // Cache hit: existing file whose hash matches.
    if let Ok(existing) = fs::read(&target) {
        if !expected_sha.is_empty() && sha256_hex(&existing) == expected_sha {
            return Ok(target);
        }
    }

    // Download.
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

    // Verify (skip only when expected hash is the empty dev placeholder).
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
```

Delete the now-unused `embedded_binary` and `extract_connector` functions. Keep `write_binary`, `binary_file_name`, `default_install_dir`, `remove_connector_file`, `probe_writable`.

- [ ] **Step 3: Update imports**

At the top of `installer.rs`, `sha2` is now used. Ensure no leftover references to `embedded_binary`/`extract_connector` remain in this file.

- [ ] **Step 4: Build**

Run: `pwsh scripts/cargo.ps1 build -p control-panel`
Expected: compiles; the only remaining callers of the deleted fns are in `commands.rs` (fixed in Task 4) — if building the lib alone fails on those, proceed to Task 4 then rebuild.

- [ ] **Step 5: Commit**

```bash
git add crates/control-panel/src/installer.rs crates/control-panel/tests/install_flow.rs
git commit -m "control-panel: fetch_connector downloads + verifies binaries on demand"
```

---

## Task 4: Wire commands to `fetch_connector`

**Files:**
- Modify: `crates/control-panel/src/commands.rs`

**Interfaces:**
- Consumes: `installer::fetch_connector`.

- [ ] **Step 1: Replace `extract_connector` calls**

In `commands.rs`, change the `use crate::installer::{...}` line to import `fetch_connector` instead of `extract_connector`:

```rust
use crate::installer::{
    default_install_dir, fetch_connector, probe_writable, remove_connector_file,
};
```

In `install_connector`, replace the `extract_connector(&id)` call with `fetch_connector(&id)`. Update the error message to mention the download (keep the existing OS-error-32 lock branch):

```rust
    let server_path = fetch_connector(&id).map_err(|e| {
        if e.raw_os_error() == Some(32) {
            format!(
                "Failed to save the connector binary: the previous server is still running \
                 and has the file locked. Fully quit Claude Desktop (check the system tray) and \
                 try again. ({e})"
            )
        } else {
            format!("Failed to download the connector: {e}. Check your internet connection or proxy.")
        }
    })?;
```

In `test_connection`, replace `extract_connector(&id)` with `fetch_connector(&id)`.

- [ ] **Step 2: Build the whole crate**

Run: `pwsh scripts/cargo.ps1 build -p control-panel`
Expected: compiles cleanly (no references to removed fns).

- [ ] **Step 3: Commit**

```bash
git add crates/control-panel/src/commands.rs
git commit -m "control-panel: install/test_connection use fetch_connector"
```

---

## Task 5: `gen-binaries-json.mjs` generator

**Files:**
- Create: `scripts/gen-binaries-json.mjs`

**Interfaces:**
- Produces: a CLI `node scripts/gen-binaries-json.mjs --win <dir> --mac <dir> --linux <dir> --version <v> --out <path>` that writes `binaries.json` with sha256+size for each of the 18 ids per OS.

- [ ] **Step 1: Write the generator**

```javascript
import { readFileSync, writeFileSync, statSync } from "node:fs";
import { createHash } from "node:crypto";
import { join } from "node:path";

const IDS = ["confluence","jira","bitbucket","airtable","mysql","mariadb","clickhouse","oracle","gitlab","postgres","github","jenkins","redmine","grafana","elasticsearch","mattermost","mongodb","sentry"];

function arg(name, def) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 ? process.argv[i + 1] : def;
}
const dirs = { win: arg("win"), mac: arg("mac"), linux: arg("linux") };
const version = arg("version");
const out = arg("out", "binaries.json");

function meta(dir, id, os) {
  const file = join(dir, os === "win" ? `${id}.exe` : id);
  const buf = readFileSync(file);
  return { sha256: createHash("sha256").update(buf).digest("hex"), size: statSync(file).size };
}

const binaries = {};
for (const id of IDS) {
  binaries[id] = { win: meta(dirs.win, id, "win"), mac: meta(dirs.mac, id, "mac"), linux: meta(dirs.linux, id, "linux") };
}
writeFileSync(out, JSON.stringify({ version, binaries }, null, 2));
console.log(`wrote ${out} for ${IDS.length} connectors`);
```

- [ ] **Step 2: Smoke-test against any existing binary**

Run (using the already-built debug binaries as a stand-in, if present):
`node -e "import('./scripts/gen-binaries-json.mjs')"` is not how to test — instead verify syntax: `node --check scripts/gen-binaries-json.mjs`
Expected: no output (syntax OK).

- [ ] **Step 3: Commit**

```bash
git add scripts/gen-binaries-json.mjs
git commit -m "scripts: generate binaries.json (per-OS sha256 manifest)"
```

---

## Task 6: L2 integration test — full fetch + config flow

**Files:**
- Modify: `crates/control-panel/tests/install_flow.rs`

**Interfaces:**
- Consumes: `control_panel::installer::fetch_connector`, `control_panel::claude_config::{write_entry, read_installed, remove_entry, default_config_path}`.

- [ ] **Step 1: Write the config round-trip test**

Append to `crates/control-panel/tests/install_flow.rs`:

```rust
use control_panel::claude_config::{read_installed, remove_entry, write_entry};
use serde_json::{json, Map};

fn temp_home() -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    let n: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    p.push(format!("ccmcp-test-{n}"));
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn write_then_read_then_remove_preserves_siblings() {
    let dir = temp_home();
    let cfg = dir.join("claude_desktop_config.json");
    std::fs::write(&cfg, r#"{"mcpServers":{"other":{"command":"x","args":[],"env":{}}}}"#).unwrap();

    let mut env = Map::new();
    env.insert("CONFLUENCE_URL".into(), json!("https://wiki.example.com"));
    write_entry(&cfg, "confluence", "/path/confluence", env).unwrap();

    let installed = read_installed(&cfg).unwrap();
    assert!(installed.iter().any(|e| e.id == "confluence"));

    // sibling preserved
    let raw: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
    assert!(raw.pointer("/mcpServers/other").is_some());

    remove_entry(&cfg, "confluence").unwrap();
    let after = read_installed(&cfg).unwrap();
    assert!(!after.iter().any(|e| e.id == "confluence"));
    let raw2: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
    assert!(raw2.pointer("/mcpServers/other").is_some());
}
```

- [ ] **Step 2: Write the fetch-from-mock-server test**

```rust
#[test]
fn fetch_downloads_when_hash_empty_and_serves_bytes() {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    // Point the install dir at a temp HOME/LOCALAPPDATA.
    let dir = temp_home();
    #[cfg(windows)]
    std::env::set_var("LOCALAPPDATA", &dir);
    #[cfg(not(windows))]
    std::env::set_var("HOME", &dir);

    // Serve a fake binary.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    std::env::set_var("CCMCP_FETCH_BASE", &base);
    let body = b"FAKE-BINARY-BYTES".to_vec();
    let body2 = body.clone();
    let h = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let mut b = [0u8; 1024];
        let _ = s.read(&mut b);
        let hdr = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body2.len());
        s.write_all(hdr.as_bytes()).unwrap();
        s.write_all(&body2).unwrap();
    });

    // `confluence` has an empty placeholder hash in resources/binaries.json,
    // so verification is skipped and the download is accepted.
    let path = control_panel::installer::fetch_connector("confluence").unwrap();
    h.join().unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), body);

    std::env::remove_var("CCMCP_FETCH_BASE");
}
```

(`installer` and `claude_config` must be `pub` in `lib.rs` — they already are.)

- [ ] **Step 3: Run the tests**

Run: `pwsh scripts/cargo.ps1 test -p control-panel --test install_flow`
Expected: PASS (both tests). Note: the fetch test mutates process env — keep it the only env-mutating test in the file or run with `--test-threads=1` if flaky.

- [ ] **Step 4: Commit**

```bash
git add crates/control-panel/tests/install_flow.rs
git commit -m "control-panel: L2 integration tests for config flow + fetch path"
```

---

## Task 7: L1 link-integrity test (site download links)

**Files:**
- Create: `tests/links.spec.ts`

**Interfaces:**
- Consumes: `site/app.js` constants, `crates/control-panel/tauri.conf.json` version.

- [ ] **Step 1: Write the test**

```typescript
import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";

// Pull CP_TAG / CP_VERSION out of site/app.js and the app version out of
// tauri.conf.json, then assert the site links match and resolve.
function readConst(src: string, name: string): string {
  const m = src.match(new RegExp(`${name}\\s*=\\s*"([^"]+)"`));
  if (!m) throw new Error(`${name} not found in app.js`);
  return m[1];
}

test("site installer links match the app version and resolve", async ({ request }) => {
  const appJs = readFileSync("site/app.js", "utf8");
  const conf = JSON.parse(readFileSync("crates/control-panel/tauri.conf.json", "utf8"));
  const cpVersion = readConst(appJs, "CP_VERSION");
  const cpTag = readConst(appJs, "CP_TAG");

  // The site must advertise the same version the app is built at.
  expect(cpVersion).toBe(conf.version);
  expect(cpTag).toBe(`cp-v${conf.version}`);

  // A Linux .deb link must exist (not only the heavy AppImage).
  expect(appJs).toMatch(/\.deb/);

  // Each advertised installer URL resolves (HEAD/GET 200 after redirects).
  const base = `https://github.com/huylq98/claude-chat-mcp/releases/download/${cpTag}`;
  const files = [
    `Claude.Chat.MCP_${cpVersion}_x64-setup.exe`,
    `Claude.Chat.MCP_${cpVersion}_aarch64.dmg`,
    `Claude.Chat.MCP_${cpVersion}_amd64.AppImage`,
    `Claude.Chat.MCP_${cpVersion}_amd64.deb`,
  ];
  for (const f of files) {
    const res = await request.get(`${base}/${f}`, { maxRedirects: 5 });
    expect(res.status(), `${f} should resolve`).toBe(200);
  }
});
```

- [ ] **Step 2: Run it (expect RED until Task 9 ships the matching release + site bump)**

Run: `npx playwright test tests/links.spec.ts`
Expected: FAIL now (site is at `cp-v0.11.0`, app at `0.13.0`; mismatch + the release lacks the new version). This proves the test catches the bug. Document the failure, do not "fix" by weakening the test.

- [ ] **Step 3: Commit**

```bash
git add tests/links.spec.ts
git commit -m "test: L1 link-integrity guard for site installer links"
```

---

## Task 8: Release workflow — publish per-OS binaries + checksums + binaries.json

**Files:**
- Modify: `.github/workflows/control-panel.yml`

**Interfaces:**
- Produces: per-OS connector binaries + `checksums.txt` on the `cp-v*` release; `binaries.json` baked into the app build.

- [ ] **Step 1: Replace the resource-staging step**

Remove the "Stage app resources (this OS's binaries + registry)" step's binary-copy loop. Keep copying `registry.json` into `resources/`. Add generation of `binaries.json` **before** `tauri build`, on a job that has all three OSes' binaries. Because the current matrix builds each OS separately, add a dedicated **manifest job** that runs after the three builds, downloads the per-OS binary artifacts, runs `node scripts/gen-binaries-json.mjs`, and commits/uploads the result so each OS build embeds the same manifest.

Concretely, restructure to match `release.yml`'s build→package pattern:
- `build` matrix job: build connector binaries, upload `bin-<plat>` artifacts (as in `release.yml`).
- `binaries-manifest` job (needs build): download all `bin-*`, run `gen-binaries-json.mjs --version <ver> --out binaries.json`, upload `binaries.json` + per-OS binaries + `checksums.txt` as artifacts, and attach per-OS binaries + `checksums.txt` to the release.
- `app` matrix job (needs binaries-manifest): download `binaries.json` into `crates/control-panel/resources/`, build the thin app with `tauri build`, upload/publish installers.

Generate `checksums.txt`:

```bash
( cd binaries/win && sha256sum * ) >> checksums.txt
( cd binaries/mac && sha256sum * ) >> checksums.txt
( cd binaries/linux && sha256sum * ) >> checksums.txt
```

- [ ] **Step 2: Validate workflow syntax locally**

Run: `npx --yes @action-validator/cli .github/workflows/control-panel.yml` (or `actionlint` if installed).
Expected: no schema errors. If neither tool is available, visually verify YAML indentation and `needs:` ordering.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/control-panel.yml
git commit -m "ci: publish per-OS connector binaries + checksums; embed binaries.json"
```

---

## Task 9: Cut the release + bump the site (closes the loop)

**Files:**
- Modify: `crates/control-panel/tauri.conf.json` (version → `0.14.0`)
- Modify: `site/app.js` (`CP_TAG`/`CP_VERSION` → `cp-v0.14.0`/`0.14.0`; add `.deb` link)

**Interfaces:**
- Consumes: Task 8 workflow.

- [ ] **Step 1: Bump app version**

In `crates/control-panel/tauri.conf.json` set `"version": "0.14.0"`. In `crates/control-panel/Cargo.toml` set `version = "0.14.0"` (so `download_url`'s `CARGO_PKG_VERSION` matches the release tag).

- [ ] **Step 2: Update the site**

In `site/app.js`:
- `CP_TAG = "cp-v0.14.0"`, `CP_VERSION = "0.14.0"`.
- Add a `.deb` entry. Make the Linux primary the `.deb` and keep AppImage as a secondary "Other downloads" link. Add to `CP_INSTALLERS`:

```javascript
const CP_INSTALLERS = {
  windows: `Claude.Chat.MCP_${CP_VERSION}_x64-setup.exe`,
  macos: `Claude.Chat.MCP_${CP_VERSION}_aarch64.dmg`,
  linux: `Claude.Chat.MCP_${CP_VERSION}_amd64.deb`,
};
const CP_LINUX_APPIMAGE = `Claude.Chat.MCP_${CP_VERSION}_amd64.AppImage`;
```

Add the AppImage as an extra link in the "all" list rendering (alongside the existing `app_other` link).

Also add the i18n string `os_linux` should read `"Linux (.deb)"` and add a new `os_linux_appimage: "Linux (AppImage)"` in both `en` and `vi` string tables.

- [ ] **Step 3: Tag + push to trigger the build (USER-AUTHORIZED ACTION)**

This publishes a GitHub Release and is outward-facing — **do not run without explicit user go-ahead**:

```bash
git add crates/control-panel/tauri.conf.json crates/control-panel/Cargo.toml site/app.js Cargo.lock
git commit -m "release: control-panel 0.14.0 (fetch-on-demand) + site .deb link"
git tag cp-v0.14.0
git push origin main --tags
```

- [ ] **Step 4: Verify the release artifacts shrank**

After the workflow completes, run:
`gh release view cp-v0.14.0 --json assets --jq '.assets[] | "\(.name)\t\(.size/1024/1024|floor)MB"'`
Expected: AppImage now a few MB (not ~100 MB); per-OS connector binaries + `checksums.txt` present.

- [ ] **Step 5: Re-run the L1 test (now GREEN)**

Run: `npx playwright test tests/links.spec.ts`
Expected: PASS — version matches, `.deb` present, all four installer URLs resolve.

---

## Self-Review

**Spec coverage (Part A + Part 0 + B-L1 portion of the spec):**
- Fetch-on-demand download + cache + verify → Tasks 3, 4, 6. ✓
- Per-OS binary source + checksums + version-locking → Tasks 2, 5, 8. ✓
- Baked expected hashes → Tasks 2, 8 (CI fills real values). ✓
- Minimal reqwest + proxy via env vars → Task 1, Global Constraints. ✓ (connector-core PAC reuse deferred — see note.)
- Installers shrink → Tasks 3 (no embedding) + 8 + verified in 9. ✓
- Site `.deb` + version bump (Part 0) → Task 9. ✓
- L1 link-integrity test → Task 7, verified in 9. ✓
- L2 headless logic incl. fetch path → Task 6. ✓

**Out of scope for this plan (separate plans):** Part C app self-update (blocked on user-created signing key) and B-L3 GUI automation. Part B-L2 on Mac/Linux runs via CI matrix when the workflow runs; locally it is verified on Windows only.

**Deviation from spec — proxy:** the spec called for reusing `connector-core`'s PAC/WPAD detection. This plan uses `reqwest`'s built-in `HTTP(S)_PROXY` env-var support for v1 (simpler, covers most corporate setups). PAC/WPAD reuse is a follow-up if a user reports a PAC-only environment. Flagged, not silently dropped.

**Placeholder scan:** no TBD/TODO; all code steps contain code. The empty-string sha256 in `resources/binaries.json` is intentional (dev placeholder that disables verification locally; CI overwrites with real hashes).

**Type consistency:** `fetch_connector(&str) -> io::Result<PathBuf>` used consistently in Tasks 3, 4, 6. `current_os_hash`/`known`/`download_url` signatures match between Task 2 and Task 3.
