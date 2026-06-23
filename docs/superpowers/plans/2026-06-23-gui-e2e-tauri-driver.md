# GUI Click-Through E2E Implementation Plan (Part B-L3)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Drive the real control-panel app through a WebDriver client to verify the click-through: launch → see connector catalog → open a connector → fill the form → Install → confirm the entry is written to `claude_desktop_config.json`. Runs on **Windows + Linux** (macOS WKWebView has no WebDriver — out of scope).

**Architecture:** `tauri-driver` bridges WebDriver to the app's webview (Linux: `WebKitWebDriver`; Windows: `msedgedriver`). A WebdriverIO test launches the built app under a temp `HOME`/`APPDATA` with `CCMCP_FETCH_BASE` pointed at a local mock binary server, so Install performs a real download+verify+config-write against throwaway paths. Isolated, **non-blocking** CI job until proven stable.

**Tech Stack:** `tauri-driver`, WebdriverIO, `WebKitWebDriver`/`xvfb` (Linux), `msedgedriver` (Windows), Node.

## Global Constraints

- macOS is unsupported by `tauri-driver` — Windows + Linux only.
- The CI job is **non-blocking** (`continue-on-error: true`) initially.
- Build via `scripts/cargo.ps1` locally (MSVC).
- The app embeds `binaries.json` where `confluence` carries the fixture hash `bfc011…f7e3` (sha256 of `FAKE-BINARY-BYTES`), so the mock server can serve those exact bytes and pass verification.
- Repo: `huylq98/claude-chat-mcp`.

---

## File Structure

- `crates/control-panel/e2e/package.json` — WebdriverIO + deps (isolated from root node_modules).
- `crates/control-panel/e2e/wdio.conf.ts` — WebdriverIO config that spawns `tauri-driver`.
- `crates/control-panel/e2e/mock-binary-server.mjs` — serves `FAKE-BINARY-BYTES` for any `/<id>-<plat>` GET.
- `crates/control-panel/e2e/specs/install.e2e.ts` — the click-through spec.
- `.github/workflows/test.yml` — add a `gui-tests` job (Windows + Linux, non-blocking).

---

## Task 1: Scaffold the WebdriverIO harness

**Files:** Create `crates/control-panel/e2e/package.json`, `wdio.conf.ts`.

- [ ] **Step 1: `package.json`** with `@wdio/cli`, `@wdio/local-runner`, `@wdio/mocha-framework`, `@wdio/spec-reporter`, `ts-node`, `typescript`.

- [ ] **Step 2: `wdio.conf.ts`** — single capability driving the built binary via `tauri-driver`:

```typescript
import { spawn, ChildProcess } from "node:child_process";
import { resolve } from "node:path";

let tauriDriver: ChildProcess;
const isWin = process.platform === "win32";
const appBinary = resolve(
  __dirname, "..",
  "target", "release",
  isWin ? "ClaudeChatMCP.exe" : "ClaudeChatMCP"
);

export const config: WebdriverIO.Config = {
  specs: ["./specs/**/*.e2e.ts"],
  maxInstances: 1,
  capabilities: [{
    // tauri-driver passes this through to the platform webdriver.
    "tauri:options": { application: appBinary },
    browserName: "wry",
  } as any],
  framework: "mocha",
  reporters: ["spec"],
  autocompileOpts: { tsNodeOpts: { transpileOnly: true } },
  onPrepare: () => {
    tauriDriver = spawn("tauri-driver", [], { stdio: [null, process.stdout, process.stderr] });
  },
  onComplete: () => { tauriDriver?.kill(); },
};
```

- [ ] **Step 3: Commit** `git add crates/control-panel/e2e/package.json crates/control-panel/e2e/wdio.conf.ts && git commit -m "control-panel e2e: WebdriverIO + tauri-driver harness"`

---

## Task 2: Mock binary server

**Files:** Create `crates/control-panel/e2e/mock-binary-server.mjs`.

- [ ] **Step 1:** A tiny HTTP server (port from argv) that returns `FAKE-BINARY-BYTES` with `Content-Length` for any GET, so `fetch_connector("confluence")` verifies against the embedded fixture hash. Reuse the pattern from `tests/install_flow.rs`'s `serve`.

- [ ] **Step 2: Commit.**

---

## Task 3: The click-through spec

**Files:** Create `crates/control-panel/e2e/specs/install.e2e.ts`.

**Interfaces:** drives DOM ids from `ui/index.html`/`ui/app.js` (verify the actual selectors: connector cards, the Configure control, the form's required input, the Install button, the saved-status text).

- [ ] **Step 1: Write the spec**

```typescript
import { readFileSync, existsSync } from "node:fs";
import { join } from "node:path";

describe("install a connector through the GUI", () => {
  it("writes the connector entry to claude_desktop_config.json", async () => {
    // The app was launched with HOME/APPDATA pointed at a temp dir (see CI/env).
    const configDir = process.env.CCMCP_TEST_CONFIG_DIR!;
    const configPath = join(configDir, "claude_desktop_config.json");

    // Catalog renders.
    const cards = await $$(".card-shell");
    expect(cards.length).toBeGreaterThan(0);

    // Open Confluence's form, fill the required URL, install.
    await (await $('[data-id="confluence"] .configure, #configure-confluence')).click();
    await (await $('input[name="CONFLUENCE_URL"], #CONFLUENCE_URL')).setValue("https://wiki.example.com");
    await (await $(".install, #install-confluence")).click();

    // Poll for the config file to gain the entry.
    await browser.waitUntil(
      () => existsSync(configPath) &&
            JSON.parse(readFileSync(configPath, "utf8"))?.mcpServers?.["claude-chat-mcp-confluence"],
      { timeout: 15000, timeoutMsg: "config entry not written" }
    );
  });
});
```

- [ ] **Step 2: Commit.**

---

## Task 4: Run locally on Windows (proves the harness)

- [ ] **Step 1:** Build the app: `pwsh scripts/cargo.ps1 build --release --manifest-path crates/control-panel/Cargo.toml` (needs `resources/registry.json` staged + `binaries.json` present).
- [ ] **Step 2:** Install `msedgedriver` matching the installed WebView2; install `tauri-driver` (`cargo install tauri-driver --locked`).
- [ ] **Step 3:** Start the mock server; set `HOME`/`APPDATA`/`LOCALAPPDATA` + `CCMCP_FETCH_BASE` + `CCMCP_TEST_CONFIG_DIR` to a temp dir; `cd crates/control-panel/e2e && npm ci && npx wdio run wdio.conf.ts`.
- [ ] **Step 4:** Expected: PASS — the temp `claude_desktop_config.json` gains `claude-chat-mcp-confluence`. Fix selectors/timing until green. **Commit** any harness fixes.

---

## Task 5: CI job (Windows + Linux, non-blocking)

**Files:** Modify `.github/workflows/test.yml`.

- [ ] **Step 1:** Add a `gui-tests` job, matrix `[windows-latest, ubuntu-latest]`, `continue-on-error: true`:
  - checkout, rust toolchain, rust-cache.
  - Linux: install `libwebkit2gtk-4.1-dev webkit2gtk-driver xvfb` + run under `xvfb-run`.
  - Windows: setup `msedgedriver` (e.g. `msedgedriver-tool` or pinned download).
  - `cargo install tauri-driver --locked`.
  - stage `resources/registry.json`; build app `--release`.
  - start mock server (background); export temp `HOME`/`APPDATA`/`LOCALAPPDATA`/`CCMCP_FETCH_BASE`/`CCMCP_TEST_CONFIG_DIR`.
  - `cd crates/control-panel/e2e && npm ci && npx wdio run wdio.conf.ts`.

- [ ] **Step 2:** Validate YAML (`python -c "import yaml; yaml.safe_load(open('.github/workflows/test.yml'))"`).

- [ ] **Step 3: Commit.**

---

## Task 6: Promote to blocking (later)

- [ ] After the job is green for several runs on both OSes, remove `continue-on-error: true`. **Commit.**

---

## Self-Review

- Harness + driver wiring → Tasks 1, 4, 5. ✓
- Real click→install→config-write E2E → Tasks 2, 3. ✓
- Windows + Linux only, non-blocking → Task 5 constraints. ✓
- **Verify-against-reality flags:** the DOM selectors in Task 3 must be confirmed against `ui/index.html`/`ui/app.js`; the `tauri:options`/`browserName` capability shape and `tauri-driver` invocation must be checked against the installed `tauri-driver` version (the WebDriver capability format has changed across versions). Both flagged inline.
- **Reliability:** GUI/WebDriver is inherently flakier than L1/L2 — hence non-blocking until proven, per the spec.
