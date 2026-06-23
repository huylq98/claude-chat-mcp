# Fetch-on-demand delivery, app self-update, and cross-OS E2E

**Date:** 2026-06-23
**Status:** Approved (design), pending implementation
**Author:** brainstormed with the user

## Problem

The Linux control-panel installer is ~100 MB (vs 7 MB on Windows) because the app
embeds all 18 connector binaries via `include_bytes!`. A ~100 MB download over a
flaky connection fails near the end — the user cannot install on Linux. Two
contributing facts confirmed against the live releases:

1. **Artifact bloat.** `cp-v0.13.0` assets: `x64-setup.exe` 7 MB, `aarch64.dmg`
   28 MB, `amd64.deb` 32 MB, **`amd64.AppImage` 100 MB**. The AppImage carries a
   full glibc/runtime payload on top of the 18 embedded binaries.
2. **Stale + incomplete site links.** `site/app.js` hard-codes `cp-v0.11.0`
   (latest is `cp-v0.13.0`), and offers Linux users only the 100 MB AppImage —
   never the 32 MB `.deb`.

There is also no automated coverage of the download → install → manage flow: the
existing Playwright suite tests only the static catalog site.

## Goals

1. Every installer is small and the connector download is small, retryable, and
   per-connector — so a dropped connection costs one retry, not a 100 MB restart.
2. The app can **check for and install its own updates** (Tauri updater).
3. A cross-OS E2E suite regression-proofs the download → install → manage flow so
   the stale-link and oversized-artifact failures cannot silently return.

## Non-goals

- Rewriting connectors or the catalog beyond the download section.
- macOS GUI automation — `tauri-driver` does not support WKWebView. Mac is
  covered by link-integrity + headless logic tests only.
- A dedicated update server. The update manifest is a static file on the
  existing site host.

---

## Part 0 — Site download fix (folded into the final release)

When the fetch-on-demand release ships, update `site/app.js`:
- Bump `CP_TAG`/`CP_VERSION` to the new release.
- Add a `.deb` link for Linux alongside the AppImage (both are small now, but the
  `.deb` is the native path for Debian/Ubuntu).

Enforced by the L1 link-integrity test (below), not by hand.

---

## Part A — Fetch-on-demand connector delivery

### Behavior change

`crates/control-panel/src/installer.rs` no longer embeds binaries. The app ships
with **zero** connector binaries; it downloads the one connector being installed,
for the current OS, on first install.

`extract_connector(id)` becomes `fetch_connector(id, progress)`:

1. **Cache check.** If `default_install_dir()/<binary_file_name(id)>` exists and
   its sha256 matches the expected hash → return it (no download).
2. **Download.** HTTPS GET the single per-OS binary, with retry/backoff and a
   progress callback surfaced to the UI.
3. **Verify.** Compute sha256 and compare to the **expected hash baked into the
   app at build time**. Mismatch → delete and error. No trust in the network
   (this app handles credentials).
4. **Install.** Cache the bytes, `chmod 0755` on Unix, then proceed exactly as
   today (`write_entry` into `claude_desktop_config.json`).

`test_connection` and `uninstall_connector` keep their current behavior;
`test_connection` calls `fetch_connector` first (same as it called
`extract_connector` before).

### Decisions (defaulted; veto-able)

- **Fetch source:** publish **per-OS raw connector binaries + `checksums.txt`**
  as assets on the matching `cp-v<version>` release. The app fetches exactly one
  small binary (~5–20 MB). Rejected alternative: downloading the `.mcpb`, which
  bundles all three OSes (≈3× larger).
- **Version locking:** the app fetches from its **own** `cp-v<version>` release,
  so app and connector binaries never drift. Updating the app (Part C)
  automatically repoints connector fetches at the new release.
- **Expected hashes:** generated in the release workflow and **baked into the app
  build** by extending the embedded `registry.json` with a
  `binaries: { <os>: { sha256, size } }` block per connector (or a sibling
  `binaries.json` embedded the same way `registry.json` is). One source of truth,
  embedded at compile time.
- **HTTP client:** add a **minimal `reqwest`** (rustls TLS) to `control-panel`.
  Reuse only `connector-core`'s proxy/PAC detection so corporate-proxy users keep
  working. Rejected alternative: pulling in `connector-core::HttpClient` (auth +
  rate-limit machinery the app does not need).
- **Failure UX:** each fetch is a small, retryable, per-connector download. On
  failure show the connector name, the error, and a Retry button. Cached after
  first success.

### CI / release changes (`.github/workflows/control-panel.yml`)

- Remove the "stage 18 binaries into `resources/`" step (the app no longer embeds
  them). `resources/registry.json` (now `registry.json` + `binaries.json`) is
  still staged.
- After building connector binaries, compute sha256 per (connector, OS), write
  `binaries.json` consumed at build time, and **upload the per-OS connector
  binaries + `checksums.txt`** to the `cp-v<version>` release.
- Result: the thin app bundles to a few MB on every OS.

---

## Part B — Cross-OS E2E suite

Runs on the existing 3-OS GitHub Actions matrix (real VMs are the sandboxes).
Docker augments the **Linux** leg only (Apple licensing forbids macOS in
containers; Windows containers have no practical WebView2 surface).

### L1 — Download / link integrity (Node, all links)

Parse the OS download URLs `site/app.js` generates and assert, for each:
- resolves with HTTP 200,
- the embedded version equals `crates/control-panel/tauri.conf.json`'s `version`,
- content-length is within sane bounds,
- the Linux `.deb` link is present.

This is the test that would have caught both current bugs.

### L2 — Headless core logic (`cargo test`, all 3 OS incl. macOS)

Integration tests that point `APPDATA` / `HOME` at a temp dir and exercise the
real functions:
- **install** → `claude_desktop_config.json` gains `claude-chat-mcp-<id>` with
  correct `command`/`env`/`<ID>_MODE`; pre-existing entries and the `.backup`
  file are preserved; a malformed config is backed up and replaced.
- **uninstall** → entry removed, sibling entries intact, cached binary deleted.
- **fetch path** → a local mock HTTP server returns a fixture binary + checksum;
  assert: checksum mismatch is rejected, a cache hit skips re-download, and the
  proxy env var is honored.

A small seam is required so the fetch base URL and config/install dirs are
injectable in tests (env-var override; `APPDATA`/`HOME` already drive the paths).

### L3 — Real GUI click-through (Windows + Linux only)

Build the thin app and drive it with `tauri-driver`:
- **Linux:** `WebKitWebDriver` + `xvfb` (runnable in Docker).
- **Windows:** `msedgedriver` matched to the WebView2 runtime.

Script: open app → pick a connector → fill the form → install → assert it appears
in the installed list and the on-disk config updated. **Isolated, non-blocking
job initially** (does not gate merges) until proven stable, because
`tauri-driver`/WebView version matching is finicky.

macOS gets no L3 (unsupported); its confidence comes from L1 + L2.

---

## Part C — App self-update ("Check for updates")

Use `tauri-plugin-updater` (+ `tauri-plugin-process` for relaunch).

### Behavior

- On startup (and via a manual **Check for updates** button) the app fetches a
  static update manifest, compares versions, and if newer offers
  version + notes + **Update now**.
- On accept: download the new signed bundle, **verify its signature**, install,
  relaunch.

### Decisions (defaulted; veto-able)

- **Manifest hosting:** a static `latest.json` published to the **existing site
  host** at a stable URL (e.g. `https://<site>/updates/latest.json`). Avoids a
  server and the `cp-v*`-not-latest tag problem. The release workflow regenerates
  and deploys it.
- **Check cadence:** check once on startup **and** on manual button press; no
  background polling.
- **Signing:** Tauri updater requires an ed25519/minisign keypair. The **public
  key is baked into `tauri.conf.json`**; the **private key + password are CI
  secrets the user must create and store** (`TAURI_SIGNING_PRIVATE_KEY`,
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`). This is a prerequisite action for the
  user — the project cannot generate/hold their signing secret.

### Release changes

- `tauri build` runs with the signing env vars → produces `.sig` files.
- Workflow generates `latest.json` (version, per-OS bundle URLs + signatures) and
  deploys it to the site updates path.

### Testing

- Headless test of the update-decision logic against a mock manifest endpoint:
  newer → update available; equal/older → none; bad signature → rejected.
- The OS-level replace+relaunch is not E2E-automated (it swaps the running
  binary); covered by manual smoke before release.

---

## Phasing

1. **A1** — fetch-on-demand in app code + L2 fetch-path tests (TDD). App stops
   embedding binaries.
2. **A2** — release workflow publishes per-OS binaries + checksums; verify with a
   test build that installers shrank.
3. **B-L1** — link-integrity test + Part 0 site update (version + `.deb`).
4. **C** — updater plugin + manifest + signing wiring + decision-logic test.
5. **B-L3** — GUI automation (non-blocking job).

Each phase is independently shippable and verifiable.

## Risks

- **`tauri-driver` setup is finicky** (driver/WebView version matching) →
  isolate L3, start non-blocking.
- **First-run offline** → fetch fails; acceptable (the user is online to download
  the app), and far better than a 100 MB up-front blob. Cached after first use.
- **Signing-secret custody** → blocks Part C until the user creates the keypair
  and stores the CI secrets. Parts A/B do not depend on it.

## Verification

- A1/A2: `cargo test` green on all 3 OS; a CI test build shows Linux installer
  dropped from ~100 MB to a few MB.
- B-L1: link-integrity job green; deliberately breaking the version in `app.js`
  makes it red.
- C: decision-logic test green; manual update smoke on one OS before release.
- B-L3: GUI job green on Windows + Linux (non-blocking until stable).
