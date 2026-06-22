//! The connector manifest consumed by the configurator UI to render auth fields
//! and register the connector. Emitted via the `--manifest` CLI flag.

/// The static manifest describing this connector and its configuration fields.
pub fn manifest() -> serde_json::Value {
    serde_json::json!({
        "id": "postgres",
        "name": "PostgreSQL",
        "group": "Data",
        "description": "Read-only SQL access to PostgreSQL.",
        "binary": "postgres",
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
            {"env": "DB_PORT", "label": "Port", "kind": "text", "required": false, "default": "5432"},
            {"env": "DB_USER", "label": "User", "kind": "text", "required": true},
            {"env": "DB_PASSWORD", "label": "Password", "kind": "secret", "required": false},
            {"env": "DB_NAME", "label": "Database/Schema", "kind": "text", "required": false}
        ],
        "notes": ""
    })
}

/// Print the manifest as pretty JSON to stdout (for `--manifest`).
pub fn print_manifest() {
    println!("{}", serde_json::to_string_pretty(&manifest()).unwrap());
}
