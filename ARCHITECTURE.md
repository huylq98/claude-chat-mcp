# Architecture

The repo is a Cargo workspace designed so that **adding connector #20 is as cheap
as adding connector #2**.

## The core insight

Claude Desktop launches **one OS process per MCP entry** in
`claude_desktop_config.json`. So instead of one mega-server multiplexing every
tool, each connector is its **own small server binary** — its own process, its
own tool namespace, its own crash domain. A bug in the Airtable connector cannot
take down Atlassian. Connectors never interfere at runtime.

## Workspace layout

```
claude-chat-mcp/
├─ crates/
│  ├─ connector-core/      # generic, connector-agnostic building blocks
│  │    • http.rs          — HttpClient: auth (Bearer/Basic), SSL/CA, proxy + PAC/WPAD,
│  │                          rate-limit semaphore, retry/backoff on 429/503
│  │    • error.rs         — CoreError      • format.rs — strip_html / truncate
│  │    • env.rs           — typed env readers
│  │    • pac.rs, system_proxy.rs — Windows proxy auto-detection
│  ├─ server-runtime/      # thin shared MCP host: init_tracing() (stderr-only) + serve_stdio()
│  └─ connectors/
│     ├─ atlassian/        # connector = one crate, one binary
│     ├─ airtable/
│     └─ database/
├─ scripts/                # cargo.ps1 (MSVC env wrapper), registry.ps1, install-local.ps1
└─ registry.json           # aggregated connector manifests (generated)
```

## What every connector crate provides (the contract)

Kept deliberately small so contributions are tractable:

1. **`config.rs`** — a `Config` loaded from env vars (+ `validate()`).
2. **`client.rs`** — API methods built on `connector_core::HttpClient` (or a
   driver, for the database connector).
3. **`handler.rs`** — the rmcp server: `#[tool_router]` with `#[tool]` methods,
   and a `#[tool_handler] impl ServerHandler` with `get_info()`.
4. **`format.rs`** — tools return **LLM-readable markdown**, never raw JSON.
5. **`manifest.rs`** — a declarative `manifest()` (id, name, group, auth fields).
   Printed via `--manifest`; aggregated into `registry.json`.
6. **`main.rs`** — `--manifest` short-circuit, else `init_tracing()` +
   `serve_stdio(Server::from_env()?)`.

## Manifest-driven, on purpose

Each connector describes its own config fields in `manifest.rs`. The (future)
GUI configurator renders auth forms from these manifests instead of hardcoding
per-connector UI, and the website reads the same `registry.json`. **One source,
two consumers** — a new connector shows up in both without UI changes.

## Scaling seams (designed-for, not built yet)

- **Installer bloat:** the GUI will embed connector binaries; at many connectors
  a single installer grows large. The manifest/registry design leaves room to
  fetch connectors on demand later.
- **Oracle / native drivers:** most connectors are pure-Rust and fully
  self-contained. Oracle is the exception (needs Instant Client) and is isolated
  behind the `oracle` cargo feature so it never affects the default build.

## Why MSVC on Windows

The project standardizes on the `x86_64-pc-windows-msvc` target (statically
linked CRT, see the Confluence-era `.cargo/config.toml` convention) because the
planned Tauri configurator needs MSVC + WebView2. `scripts/cargo.ps1` loads the
VS Build Tools dev environment so `link.exe` and the Windows SDK are on PATH.
