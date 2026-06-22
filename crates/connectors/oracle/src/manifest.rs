//! The connector manifest consumed by the configurator UI to render auth fields
//! and register the connector. Emitted via the `--manifest` CLI flag.

/// The static manifest describing this connector and its configuration fields.
pub fn manifest() -> serde_json::Value {
    serde_json::json!({
        "id": "oracle",
        "name": "Oracle",
        "group": "Data",
        "description": "Read-only SQL access to an Oracle database.",
        "binary": "oracle",
        "docs_url": "",
        "tools": [
            {"name": "list_databases", "description": "List databases / schemas on the server."},
            {"name": "list_tables", "description": "List tables in a database."},
            {"name": "describe_table", "description": "Show a table's columns and types."},
            {"name": "run_query", "description": "Run a read-only SQL query (guarded)."},
            {"name": "server_info", "description": "Show the engine and connection summary."}
        ],
        "auth_fields": [
            {"env": "DB_HOST", "label": "Host", "kind": "text", "required": true, "default": "127.0.0.1"},
            {"env": "DB_PORT", "label": "Port", "kind": "text", "required": false, "default": "1521"},
            {"env": "DB_USER", "label": "User", "kind": "text", "required": true},
            {"env": "DB_PASSWORD", "label": "Password", "kind": "secret", "required": false},
            {"env": "DB_NAME", "label": "Database/Schema", "kind": "text", "required": false},
            {"env": "DB_SERVICE", "label": "Service Name", "kind": "text", "required": false}
        ],
        "notes": "Requires building with --features oracle and Oracle Instant Client installed."
    })
}

/// Print the manifest as pretty JSON to stdout (for `--manifest`).
pub fn print_manifest() {
    println!("{}", serde_json::to_string_pretty(&manifest()).unwrap());
}
