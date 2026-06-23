# App Self-Update Implementation Plan (Part C)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Let the desktop app check for, download, verify, and install newer versions of itself ("Check for updates"), using Tauri v2's updater plugin against a static manifest hosted on the site.

**Architecture:** Add `tauri-plugin-updater` + `tauri-plugin-process`. On startup and on a manual button, the app fetches `https://<site>/updates/latest.json`, compares versions, and if newer offers download → signature-verify → install → relaunch. The release workflow signs the bundles and regenerates `latest.json`.

**Tech Stack:** Tauri v2 updater/process plugins, ed25519/minisign signing, static JSON manifest, GitHub Actions.

## Global Constraints

- App version source of truth: `crates/control-panel/tauri.conf.json` `version`.
- Build on this machine only via `scripts/cargo.ps1` (MSVC dev-shell).
- Updater signing keypair: **public key** committed in `tauri.conf.json`; **private key + password** are CI secrets (`TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`) the user creates — never commit the private key.
- Manifest URL is stable and HTTPS.
- Repo: `huylq98/claude-chat-mcp`.

## PREREQUISITE (user action — blocks everything below)

Generate the updater keypair and store the secrets. Run locally:

```bash
cargo install tauri-cli --version "^2.0" --locked
cargo tauri signer generate -w ~/.tauri/ccmcp-updater.key
```

This prints a **public key** (used in Task 2) and writes a password-protected **private key**. Add two GitHub repo secrets: `TAURI_SIGNING_PRIVATE_KEY` (contents of the `.key` file) and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Keep the private key offline; losing it means users can never auto-update again.

---

## File Structure

- `crates/control-panel/Cargo.toml` — add `tauri-plugin-updater`, `tauri-plugin-process`.
- `crates/control-panel/src/lib.rs` — register both plugins.
- `crates/control-panel/tauri.conf.json` — add `plugins.updater` (endpoints + pubkey); add `createUpdaterArtifacts: true` to bundle config.
- `crates/control-panel/capabilities/default.json` — allow `updater:default`, `process:default`.
- `crates/control-panel/ui/index.html` + `ui/app.js` + `ui/style.css` — "Check for updates" button + update dialog.
- `site/updates/latest.json` — static update manifest (generated/deployed by CI; committed placeholder for first deploy).
- `.github/workflows/control-panel.yml` — sign bundles (env secrets) + regenerate `latest.json` + deploy it.
- `scripts/gen-update-manifest.mjs` — build `latest.json` from the signed bundle `.sig` files.

---

## Task 1: Add updater + process plugins

**Files:** Modify `crates/control-panel/Cargo.toml`.

- [ ] **Step 1: Add dependencies**

```toml
tauri-plugin-updater = "2"
tauri-plugin-process = "2"
```

- [ ] **Step 2: Build**

Run: `pwsh scripts/cargo.ps1 build --manifest-path crates/control-panel/Cargo.toml`
Expected: compiles.

- [ ] **Step 3: Commit**

```bash
git add crates/control-panel/Cargo.toml crates/control-panel/Cargo.lock
git commit -m "control-panel: add updater + process plugins"
```

---

## Task 2: Configure the updater (manifest endpoint + pubkey)

**Files:** Modify `crates/control-panel/tauri.conf.json`, `crates/control-panel/capabilities/default.json`.

- [ ] **Step 1: Add the updater plugin config**

In `tauri.conf.json`, add a top-level `plugins` block (replace `<PUBKEY>` with the public key from the prerequisite):

```json
"plugins": {
  "updater": {
    "endpoints": ["https://claude-chat-mcp.pages.dev/updates/latest.json"],
    "pubkey": "<PUBKEY>"
  }
}
```

And in `bundle`, add:

```json
"createUpdaterArtifacts": true
```

(Verify the exact site host; adjust the endpoint to the deployed domain.)

- [ ] **Step 2: Grant capabilities**

In `capabilities/default.json`, add to the `permissions` array:

```json
"updater:default",
"process:default"
```

- [ ] **Step 3: Build to validate config**

Run: `pwsh scripts/cargo.ps1 build --manifest-path crates/control-panel/Cargo.toml`
Expected: compiles (config is validated at build).

- [ ] **Step 4: Commit**

```bash
git add crates/control-panel/tauri.conf.json crates/control-panel/capabilities/default.json
git commit -m "control-panel: configure updater endpoint + capabilities"
```

---

## Task 3: Register plugins + expose a check command

**Files:** Modify `crates/control-panel/src/lib.rs`. Create `crates/control-panel/src/update.rs`.

**Interfaces:**
- Produces: `#[tauri::command] async fn check_update(app) -> Result<Option<UpdateInfo>, String>` and `async fn install_update(app) -> Result<(), String>`, where `UpdateInfo { version: String, notes: String }`.

- [ ] **Step 1: Write `update.rs`**

```rust
use serde::Serialize;
use tauri_plugin_updater::UpdaterExt;

#[derive(Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
}

/// Returns Some(info) if a newer signed version is available, else None.
#[tauri::command]
pub async fn check_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await.map_err(|e| e.to_string())? {
        Some(update) => Ok(Some(UpdateInfo {
            version: update.version.clone(),
            notes: update.body.clone().unwrap_or_default(),
        })),
        None => Ok(None),
    }
}

/// Downloads + verifies + installs the pending update, then relaunches.
#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    if let Some(update) = updater.check().await.map_err(|e| e.to_string())? {
        update
            .download_and_install(|_chunk, _total| {}, || {})
            .await
            .map_err(|e| e.to_string())?;
        app.restart();
    }
    Ok(())
}
```

(Verify `update.version` / `update.body` field names against the installed `tauri-plugin-updater` docs; adjust if the API differs.)

- [ ] **Step 2: Register in `lib.rs`**

Add `pub mod update;`, the plugins, and the commands:

```rust
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_connectors,
            commands::list_installed,
            commands::install_connector,
            commands::test_connection,
            commands::uninstall_connector,
            update::check_update,
            update::install_update,
        ])
```

- [ ] **Step 3: Build**

Run: `pwsh scripts/cargo.ps1 build --manifest-path crates/control-panel/Cargo.toml`
Expected: compiles.

- [ ] **Step 4: Commit**

```bash
git add crates/control-panel/src/update.rs crates/control-panel/src/lib.rs
git commit -m "control-panel: check_update/install_update commands"
```

---

## Task 4: UI — "Check for updates" button + dialog

**Files:** Modify `crates/control-panel/ui/index.html`, `ui/app.js`, `ui/style.css`.

- [ ] **Step 1: Add the button to the header**

In `index.html` near the title/header, add:

```html
<button id="check-update" class="link-btn"></button>
<div id="update-banner" hidden></div>
```

- [ ] **Step 2: Add strings + wiring in `app.js`**

Add to both `en` and `vi` STRINGS:

```javascript
checkUpdate: "Check for updates",        // vi: "Kiểm tra cập nhật"
checkingUpdate: "Checking…",             // vi: "Đang kiểm tra…"
upToDate: "You're on the latest version.", // vi: "Bạn đang dùng bản mới nhất."
updateAvail: "Update available:",        // vi: "Có bản cập nhật:"
updateNow: "Update now",                 // vi: "Cập nhật ngay"
updating: "Updating…",                   // vi: "Đang cập nhật…"
```

Wire the button + an on-startup check:

```javascript
async function checkForUpdate(manual) {
  const banner = document.getElementById("update-banner");
  try {
    if (manual) setStatus(t("checkingUpdate"));
    const info = await invoke("check_update");
    if (!info) { if (manual) setStatus(t("upToDate")); return; }
    banner.hidden = false;
    banner.innerHTML =
      `<span>${esc(t("updateAvail"))} ${esc(info.version)}</span>` +
      `<button id="do-update" class="btn btn-primary">${esc(t("updateNow"))}</button>`;
    document.getElementById("do-update").onclick = async () => {
      setStatus(t("updating"));
      await invoke("install_update");
    };
  } catch (e) {
    if (manual) setStatus(String(e));
  }
}
document.getElementById("check-update").addEventListener("click", () => checkForUpdate(true));
window.addEventListener("DOMContentLoaded", () => checkForUpdate(false));
```

(Use the page's existing `setStatus`/`esc`/`t` helpers; match their actual names.)

- [ ] **Step 3: Style the button + banner** in `style.css` (match existing tokens).

- [ ] **Step 4: Commit**

```bash
git add crates/control-panel/ui/index.html crates/control-panel/ui/app.js crates/control-panel/ui/style.css
git commit -m "control-panel: Check-for-updates UI"
```

---

## Task 5: Update manifest generator + static placeholder

**Files:** Create `scripts/gen-update-manifest.mjs`, `site/updates/latest.json`.

**Interfaces:** `node scripts/gen-update-manifest.mjs --version <v> --bundles <dir> --notes <str> --pub-date <iso> --out site/updates/latest.json` producing Tauri's expected manifest:

```json
{
  "version": "0.15.0",
  "notes": "…",
  "pub_date": "2026-…Z",
  "platforms": {
    "windows-x86_64": { "signature": "<sig>", "url": "https://github.com/huylq98/claude-chat-mcp/releases/download/cp-v0.15.0/Claude.Chat.MCP_0.15.0_x64-setup.exe" },
    "darwin-aarch64": { "signature": "<sig>", "url": "…aarch64.app.tar.gz" },
    "darwin-x86_64":  { "signature": "<sig>", "url": "…x64.app.tar.gz" },
    "linux-x86_64":   { "signature": "<sig>", "url": "…amd64.AppImage" }
  }
}
```

- [ ] **Step 1: Write the generator** — read each platform's `.sig` file (next to the bundle), map to Tauri target keys, write the manifest. (`pub_date` passed in, not generated, for reproducibility.)

- [ ] **Step 2: Commit a placeholder** `site/updates/latest.json` for the current version so the endpoint resolves on first deploy.

- [ ] **Step 3: Commit**

```bash
git add scripts/gen-update-manifest.mjs site/updates/latest.json
git commit -m "scripts: update-manifest generator + placeholder"
```

---

## Task 6: Sign bundles + deploy the manifest in CI

**Files:** Modify `.github/workflows/control-panel.yml`.

- [ ] **Step 1:** In the `app` job, pass signing env to `cargo tauri build`:

```yaml
      - name: Build app + installer
        working-directory: crates/control-panel
        env:
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        run: cargo tauri build
```

Include the generated `.sig` files + updater artifacts (`*.app.tar.gz`, `*.AppImage`, NSIS) in the upload/publish `files:` globs.

- [ ] **Step 2:** Add a job (needs app) that downloads the published bundles + `.sig`, runs `gen-update-manifest.mjs`, and deploys `latest.json` to the site host (Cloudflare Pages via `wrangler`, matching the existing site deploy).

- [ ] **Step 3:** Validate YAML: `python -c "import yaml; yaml.safe_load(open('.github/workflows/control-panel.yml'))"`.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/control-panel.yml
git commit -m "ci: sign updater bundles + deploy latest.json"
```

---

## Task 7: Headless update-decision test

**Files:** `crates/control-panel/tests/update_decision.rs`.

- [ ] **Step 1:** Serve a mock `latest.json` from a local server (reuse the `serve` helper pattern from `install_flow.rs`); assert the version-comparison logic treats a higher manifest version as "available", an equal/lower one as "none". (The signature path is covered by the plugin; this test guards our endpoint/version handling.) Because `check()` requires a real `AppHandle`, factor the version-compare into a pure helper `is_newer(current, candidate) -> bool` in `update.rs` and unit-test that directly.

- [ ] **Step 2:** Run: `pwsh scripts/cargo.ps1 test --manifest-path crates/control-panel/Cargo.toml is_newer`
Expected: PASS.

- [ ] **Step 3: Commit.**

---

## Self-Review

- Updater plugin + config + commands + UI → Tasks 1–4. ✓
- Manifest generation + hosting → Tasks 5, 6. ✓
- Signing wiring (CI secrets) → Task 6 + Prerequisite. ✓
- Test (version-decision) → Task 7. ✓
- **Blocked-on-user:** signing keypair (Prerequisite) — without it, Task 6 builds unsigned and the updater refuses to install.
- **Verify-against-docs flags:** plugin API field names in Task 3 and the manifest target keys in Task 5 must be checked against the installed `tauri-plugin-updater` version (it evolves); flagged inline.
- The OS-level replace+relaunch is not automated; smoke-test manually on one OS before release.
