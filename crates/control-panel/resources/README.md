# Compile-time resources (populated by the lead)

These files are embedded into the binary at compile time via `include_bytes!`
in `src/installer.rs`. They MUST exist before `cargo build` / `cargo tauri build`.

## Required connector binaries

One release binary per connector id. On Windows use the `.exe` suffix; on
other platforms use the bare name.

Windows:
- `confluence.exe`
- `jira.exe`
- `bitbucket.exe`
- `airtable.exe`
- `mysql.exe`
- `mariadb.exe`
- `clickhouse.exe`
- `oracle.exe`

macOS / Linux (bare names, same ids):
- `confluence`, `jira`, `bitbucket`, `airtable`, `mysql`, `mariadb`, `clickhouse`, `oracle`

## Registry

`src/registry.rs` embeds the catalog via
`include_str!("../resources/registry.json")` (this file:
`crates/control-panel/resources/registry.json`). It is a copy of the repo-root
`site/registry.json`; keep the two in sync whenever the catalog changes.

(The spec asked for `include_str!("../../site/registry.json")`, but relative to
`src/` that resolves to the nonexistent `crates/site/registry.json`, so the
embed was repointed here.)
