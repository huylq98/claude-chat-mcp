# Claude Chat MCP

**A free, open-source collection of local connectors that link Claude Desktop to the self-hosted and enterprise tools your team actually runs.**

Anthropic's built-in connectors mostly target the public cloud editions of tools (Confluence Cloud, GitHub.com, and so on). But many teams live on the self-hosted, Data Center, and on-prem editions running behind a company firewall, plus tools Claude does not support at all. This project fills that gap.

Each connector is a small, self-contained MCP server that Claude Desktop launches on your own machine. There is no cloud service, no hosting to set up, and it works behind corporate proxies. Your credentials stay on your computer and are only ever sent to your own server.

Open source under the [MIT license](LICENSE). Built for Windows, macOS, and Linux.

## Connectors

17 connectors, grouped by what they connect to. Each one is its own standalone server.

### Atlassian

| Connector | What it does |
|---|---|
| Confluence | Search and read self-hosted Confluence pages and spaces (Server / Data Center). |
| Jira | Search and read self-hosted Jira issues and projects with JQL (Server / Data Center). |
| Bitbucket | Browse self-hosted Bitbucket repositories, pull requests, and commits (Server / Data Center). |

### Dev

| Connector | What it does |
|---|---|
| GitHub | Search and read GitHub Enterprise Server repos, issues, and pull requests. Can create issues and comments in Writer mode. |
| GitLab | Search and read self-hosted GitLab projects, issues, and merge requests. Can create issues and comments in Writer mode. |
| Jenkins | Browse self-hosted Jenkins jobs and builds. Can trigger builds in Writer mode. |
| Grafana | Browse self-hosted Grafana dashboards, data sources, and alerts. Can add annotations in Writer mode. |

### Productivity

| Connector | What it does |
|---|---|
| Airtable | List bases, tables, and records, and read individual records. Can create and update records in Writer mode. |
| Redmine | Search and read self-hosted Redmine projects and issues. Can create issues and notes in Writer mode. |
| Mattermost | Read self-hosted Mattermost teams, channels, and messages. Can post messages in Writer mode. |

### Data

| Connector | What it does |
|---|---|
| PostgreSQL | Read-only SQL: list databases, tables, and columns, and run guarded queries. |
| MySQL | Read-only SQL over MySQL (same tools as above). |
| MariaDB | Read-only SQL over MariaDB. |
| ClickHouse | Read-only SQL over ClickHouse (HTTP interface). |
| Oracle | Read-only SQL over Oracle. Requires a special build (see note below). |
| Elasticsearch | Search self-hosted Elasticsearch or OpenSearch indices and read mappings. Can index documents in Writer mode. |
| MongoDB | Query self-hosted MongoDB databases and collections. Can insert documents in Writer mode. |

> **Oracle note:** Oracle has no pure-Rust driver, so it is built behind an `oracle` cargo feature and needs Oracle Instant Client installed at runtime. The other sixteen connectors are fully self-contained with nothing extra to install.

## Install

There are two ways to add a connector. Most people should use the control panel app.

### Option A: The control panel app (recommended)

A small desktop app that does everything for you: it bundles the connectors, lets you fill in your details in a form, tests the connection, and writes the Claude Desktop config itself.

1. Go to the [Releases page](https://github.com/huylq98/claude-chat-mcp/releases) and download the installer for your operating system:
   - **Windows:** the `.msi` or `-setup.exe` file
   - **macOS:** the `.dmg` file
   - **Linux:** the `.deb` or `.AppImage` file
2. Run the installer and open **Claude Chat MCP**.
3. Pick a connector and click **Configure**.
4. Enter your details:
   - The **link** to your server (for example `https://wiki.corp.com`).
   - Your **credentials** (usually a personal access token or API key; username and password also work for some connectors).
   - Optionally a **proxy** or other network settings under **Advanced** (leave blank unless your IT team tells you otherwise).
5. Choose a **permission role**: Viewer or Writer (see [Permissions](#permissions) below).
6. Click **Test connection** to confirm it works, then **Install**.
7. **Fully quit Claude Desktop** (on Windows, right-click the tray icon and choose Exit) and reopen it.

Now ask Claude things like *"Search our Confluence for the deployment runbook"* or *"List the tables in the app database."*

The app is also available in English and Vietnamese, and lets you update or remove connectors later from the same screen.

> **A note on installers:** the installers are not code-signed yet. Windows SmartScreen or macOS Gatekeeper may warn you the first time you run them. This is expected for an unsigned open-source app.
>
> **WebView2 on Windows:** the app uses Microsoft's WebView2 runtime, which ships with Windows 11 and recent Windows 10. If it is missing, the app will point you to the free download.

### Option B: Direct `.mcpb` bundle (advanced)

If you would rather not install the control panel app, each connector is also published as a standalone `.mcpb` bundle that Claude Desktop can load directly.

1. On the [Releases page](https://github.com/huylq98/claude-chat-mcp/releases), download the `.mcpb` file for the connector you want. Each bundle is universal and works on Windows, macOS, and Linux.
2. In Claude Desktop, open **Settings > Extensions**.
3. Double-click the `.mcpb` file, or drag it into that Extensions screen.
4. Fill in the connector's configuration fields (link, credentials, and so on).
5. Fully quit and reopen Claude Desktop.

## Permissions

Every connector runs in one of two roles, which you choose at install time:

- **Viewer (read only):** Claude can read your data but cannot change anything. This is the safe default. Choose it unless you specifically need Claude to make changes.
- **Writer (read and write):** Claude can also create, update, and delete data through the connector, for example creating a Jira-style issue or updating an Airtable record.

The read-only Data connectors are always read only regardless of role. For the others, write tools are only enabled in Writer mode.

## Privacy

- You supply your own credentials at install time. Nothing is bundled with the connectors and nothing is hard-coded.
- Connectors run entirely on your own machine and only talk to the server address you provide. No data is sent to this project or any third party.
- Credentials are stored by Claude Desktop in its own local config file, the same way any MCP server's settings are.

## Build from source

For developers who want to build the connectors themselves.

This is a Rust workspace. You need the stable Rust toolchain.

```bash
cargo build --release          # build all connector binaries
cargo test                     # run the test suite
```

On Windows, the connectors link with MSVC, so you need the Visual Studio Build Tools (C++ workload and the Windows SDK). The repo ships a wrapper that loads the MSVC environment for you:

```powershell
./scripts/cargo.ps1 build
./scripts/cargo.ps1 test
```

Each connector binary supports a couple of helper flags:

- `--manifest` prints the connector's config descriptor (the fields shown in the UI).
- `--test-connection` checks the credentials in the environment and exits.

The desktop control panel is a [Tauri](https://tauri.app/) app that lives in `crates/control-panel/`. It is **excluded from the main Cargo workspace** and built separately with `cargo tauri build`. It embeds the connector binaries, so build those first. See the build steps in `.github/workflows/control-panel.yml` for the exact sequence the CI uses.

## How it works

Claude Desktop launches one OS process per MCP entry, so this project ships one small server binary per connector. Each has its own process, its own tool namespace, and its own crash domain. A bug in one connector cannot take down another.

A shared `connector-core` crate handles the common plumbing (HTTP client, authentication, proxy and PAC support, retries), and `server-runtime` provides the stdio MCP host. Adding a connector means adding one small crate.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full design.

## Contributing

Contributions are welcome, especially new connectors. The repo is built so that adding a connector is mostly copy, paste, and fill in. See [CONTRIBUTING.md](CONTRIBUTING.md) for the step-by-step guide.

## License

[MIT](LICENSE)
