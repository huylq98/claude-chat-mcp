# Contributing

The whole point of this repo is breadth: many connectors, each one small. Adding
a connector is intentionally a copy-paste-and-fill exercise.

## Add a connector in 5 steps

1. **Copy a crate.** `crates/connectors/airtable/` is the simplest reference
   (single HTTP base URL, Bearer auth). Copy it to
   `crates/connectors/<your-connector>/` and rename in `Cargo.toml` (`name`,
   `[[bin]] name`).

2. **Register it** in the root `Cargo.toml` `members` list.

3. **Fill in the four files:**
   - `config.rs` — your env vars + `validate()`.
   - `client.rs` — API methods on `connector_core::HttpClient`
     (`get_json` / `send_json` / `send_empty` / `get_text` / `post_text`).
   - `handler.rs` — one `#[tool]` method per capability; return markdown via
     `format.rs`. Surface API errors as tool text, don't fail the call.
   - `manifest.rs` — describe your auth fields (this drives the future wizard UI).

4. **Build & smoke-test:**
   ```powershell
   ./scripts/cargo.ps1 build -p <your-connector>
   ./target/debug/<your-connector>.exe --manifest        # prints the descriptor
   ```
   For a full MCP check, register it with `scripts/install-local.ps1` and ask
   Claude to use it.

5. **Add tests** for your formatters / config parsing (see `airtable` and
   `database` for examples) and open a PR.

## Conventions

- Tools return **LLM-readable markdown**, not raw JSON.
- **Never log to stdout** — stdout is the MCP wire protocol. Use `tracing`
  (goes to stderr via `server_runtime::init_tracing`).
- Read-only by default where it matters (see the `database` connector's
  `guard_read_only`); make write tools explicit.
- Credentials come from env vars at runtime and are never committed. `.env` is
  gitignored.

## Native-dependency connectors

If a connector needs a non-Rust driver (like Oracle's Instant Client), gate it
behind a cargo `feature` so it never breaks the default build, and document the
runtime requirement in its `manifest.rs` `notes` and the README table.
