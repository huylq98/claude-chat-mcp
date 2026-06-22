//! The connector manifest consumed by the configurator UI to render auth fields
//! and register the connector. Emitted via the `--manifest` CLI flag.

/// The static manifest describing this connector and its configuration fields.
pub fn manifest() -> serde_json::Value {
    serde_json::json!({
        "id": "mongodb",
        "name": "MongoDB",
        "group": "Data",
        "description": "Query self-hosted MongoDB databases and collections (and insert documents in Writer mode).",
        "binary": "mongodb",
        "docs_url": "https://www.mongodb.com/docs/drivers/rust/current/",
        "tools": [
            {"name": "mongo_list_databases", "description": "List the names of all databases on the server."},
            {"name": "mongo_list_collections", "description": "List the collection names in a database (defaults to the configured database)."},
            {"name": "mongo_find", "description": "Find documents in a collection matching an optional JSON filter, up to a limit."},
            {"name": "mongo_count", "description": "Count documents in a collection matching an optional JSON filter."},
            {"name": "mongo_insert", "description": "Insert a single JSON document into a collection (Writer mode only)."}
        ],
        "auth_fields": [
            {"env": "MONGODB_URI", "label": "Connection URI", "kind": "secret", "required": true, "help": "e.g. mongodb://user:pass@host:27017"},
            {"env": "MONGODB_DATABASE", "label": "Default Database", "kind": "text", "required": false, "help": "Default database for collection operations"}
        ],
        "advanced_fields": [],
        "notes": ""
    })
}

/// Print the manifest as pretty JSON to stdout (for `--manifest`).
pub fn print_manifest() {
    println!("{}", serde_json::to_string_pretty(&manifest()).unwrap());
}
