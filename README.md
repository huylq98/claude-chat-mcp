# Claude Chat MCP

**Connect Claude Desktop to the self-hosted / enterprise tools your office actually uses.**

Anthropic's official connectors mostly cover the *public cloud* editions of tools (Confluence Cloud, etc.). But corporate users live in the **self-hosted / Data Center / on-prem** editions their company runs behind a firewall — and in tools Claude doesn't support at all. This project is a growing collection of local MCP connectors that fill exactly that gap.

Each connector is a small, self-contained MCP server that Claude Desktop launches over stdio. No cloud, no hosting, works behind corporate proxies.

> Open-source (MIT). Cross-platform target: **Windows, macOS, Linux**. Windows ships as a proper installer (planned — see Roadmap).

## Connectors

Each tool/service is its **own** standalone connector (its own binary and process):

| Connector | Group | What it connects |
|---|---|---|
| **confluence** | Atlassian | Self-hosted Confluence (Server / Data Center): search + read pages |
| **jira** | Atlassian | Self-hosted Jira: search issues (JQL) + read |
| **bitbucket** | Atlassian | Bitbucket Server: repos, pull requests, commits |
| **airtable** | Productivity | Airtable bases, tables, and records (read + create/update) |
| **mysql** | Data | Read-only SQL over MySQL |
| **mariadb** | Data | Read-only SQL over MariaDB |
| **clickhouse** | Data | Read-only SQL over ClickHouse |
| **oracle** | Data | Read-only SQL over Oracle¹ |

¹ Oracle is gated behind the `oracle` build feature and requires Oracle Instant Client (it has no pure-Rust driver). The others are fully self-contained.

Run any connector with `--manifest` to see its config fields; `registry.json` aggregates all of them, and `scripts/serve-site.ps1` serves a bilingual (EN/VI) catalog site at http://localhost:4321.

## Build (local, for review)

Requires the Rust toolchain (stable) and, on Windows, **Visual Studio Build Tools with the C++ workload + Windows SDK** (the MSVC linker). The repo's wrapper loads the MSVC dev environment for you:

```powershell
./scripts/cargo.ps1 build            # debug build of all connectors
./scripts/cargo.ps1 test             # run the test suite
```

On macOS/Linux you can use `cargo` directly (no wrapper needed).

Binaries land in `target/debug/{atlassian,airtable,database}.exe`.

## Try it in Claude Desktop

Until the GUI configurator ships, register a built connector with your local Claude Desktop:

```powershell
# Confluence (self-hosted)
./scripts/install-local.ps1 confluence @{ CONFLUENCE_URL="https://wiki.corp.com"; CONFLUENCE_TOKEN="<your PAT>" }

# MySQL (read-only)
./scripts/install-local.ps1 mysql @{ DB_HOST="127.0.0.1"; DB_USER="root"; DB_PASSWORD="secret"; DB_NAME="app" }

# Airtable
./scripts/install-local.ps1 airtable @{ AIRTABLE_TOKEN="<your PAT>" }
```

> For non-technical users, the GUI installer (roadmap) replaces these commands with a click-through wizard. The `scripts/` path is the developer/reviewer route.

Then fully quit Claude Desktop (tray → Exit) and reopen it. Ask Claude things like
*"Search our Confluence for the deployment runbook"* or *"List the tables in the app database."*

To do it by hand instead, add an entry to `claude_desktop_config.json` →
`mcpServers` with `command` = the binary path and `env` = the connector's variables.

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md). In short: a shared `connector-core` (HTTP
client, auth, proxy/PAC, retry) and `server-runtime` (stdio MCP host), with **one
small server binary per connector**. Adding a connector means adding one crate —
see [CONTRIBUTING.md](CONTRIBUTING.md).

## Roadmap

- [ ] GUI configurator (Tauri) — manifest-driven, multi-connector, writes the Claude Desktop config; Windows installer (MSI/NSIS), macOS `.pkg`/`.dmg`, Linux `.deb`/`.AppImage`.
- [ ] More connectors (GitLab self-hosted, ServiceNow, internal APIs, …).
- [ ] Companion website / SEO content hub generated from `registry.json`.

## License

[MIT](LICENSE)
