# Claude Chat MCP — Design

Date: 2026-06-22
Status: First increment shipped (local build verified)

## Problem & wedge

Anthropic's official Claude connectors mostly target the public/cloud editions of
tools. Office workers run **self-hosted / Data Center / on-prem** editions behind
firewalls — or tools Claude doesn't support at all. The Confluence connector
(`confluence-mcp-server`) proved demand by targeting Confluence **Server/DC**.
This project generalizes that into a **collection of local MCP connectors** for
self-hosted/enterprise tools, delivered to **Claude Desktop** (local stdio).

## Decisions (from brainstorming)

- **Product shape:** multi-connector product (not a directory or hosted service).
- **Target surface:** Claude Desktop, local stdio `.exe` per connector.
- **Monetization:** free + open-source (MIT); revenue, if any, later via an SEO
  content site generated from the connector registry. No paywall plumbing now.
- **Repo:** new repo `claude-chat-mcp`, seeded from the Confluence code's generic
  pieces. Open-sourced.
- **Platforms:** Windows + macOS + Linux. Windows ships as an installer (planned).
- **First increment:** generalize the framework + ship Atlassian, Airtable, and a
  multi-engine Database connector.

## Architecture

One Cargo workspace. Shared `connector-core` (HTTP client: auth, SSL/CA,
proxy/PAC, rate-limit, retry; format & env helpers) and `server-runtime` (stdio
MCP host: stderr logging + serve loop). **One server binary per connector** —
Claude Desktop runs one process per MCP entry, so this gives natural isolation.
Each connector is one crate providing config / client / handler / format /
manifest. Manifests are emitted via `--manifest` and aggregated into
`registry.json` — the single source the future wizard and website both read.

See `ARCHITECTURE.md` for detail.

## Shipped in this increment

| Connector | Tools | Notes |
|---|---|---|
| atlassian | 9 | Confluence + Jira + Bitbucket, Data Center REST, shared PAT/basic auth, per-product base URLs |
| airtable | 6 | PAT bearer; list/get/create/update records, list bases/tables |
| database | 5 | Engine trait; MySQL/MariaDB (`mysql_async`), ClickHouse (HTTP). Read-only guard. Oracle behind `oracle` feature (needs Instant Client) |

Verified: `cargo test` green (32 tests); real MCP `initialize` + `tools/list`
handshake against all three binaries; `--manifest` and no-config error paths.

## Deferred (future increments, designed-for not built)

- Tauri GUI configurator (manifest-driven) + OS installers (MSI/NSIS, pkg/dmg, deb/AppImage).
- On-demand connector download (avoid installer bloat at many connectors).
- More connectors (self-hosted GitLab, ServiceNow, internal APIs).
- SEO content website generated from `registry.json`.

## Build environment note

Local build was blocked by a missing Windows SDK (only the MSVC compiler/linker
were present). Resolved by adding the Windows 11 SDK to VS Build Tools.
`scripts/cargo.ps1` loads the MSVC dev environment so builds "just work".
