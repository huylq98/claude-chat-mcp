//! Declarative description of this connector for a future configuration wizard
//! / registry. Emitted as JSON when the binary is run with `--manifest`.

use serde_json::Value;

/// Machine-readable manifest describing the connector's identity and the env
/// vars a configurator should prompt for.
pub fn manifest() -> Value {
    serde_json::json!({
        "id": "airtable",
        "name": "Airtable",
        "group": "Productivity",
        "description": "Read and write Airtable bases, tables, and records.",
        "binary": "airtable",
        "docs_url": "https://airtable.com/developers/web/api/introduction",
        "tools": [
            {"name": "list_bases", "description": "List the bases the token can access."},
            {"name": "list_tables", "description": "List tables and fields in a base."},
            {"name": "list_records", "description": "List records, with view and formula filters."},
            {"name": "get_record", "description": "Fetch a single record by ID."},
            {"name": "create_record", "description": "Create a record in a table."},
            {"name": "update_record", "description": "Update fields on an existing record."}
        ],
        "auth_fields": [
            {
                "env": "AIRTABLE_TOKEN",
                "label": "Personal Access Token",
                "kind": "secret",
                "required": true,
                "help": "Create at https://airtable.com/create/tokens with data.records:read scope (and write if needed)."
            }
        ],
        "advanced_fields": [
            {"env": "AIRTABLE_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "AIRTABLE_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ]
    })
}

/// Print the manifest as pretty JSON to stdout. Called when the binary is run
/// with `--manifest` (before any tracing/stdio setup).
pub fn print_manifest() {
    match serde_json::to_string_pretty(&manifest()) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("failed to serialize manifest: {e}"),
    }
}
