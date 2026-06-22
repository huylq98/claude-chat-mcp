//! Declarative description of this connector, consumed by the (future)
//! configurator wizard and the website registry. Emitted as JSON when the
//! binary is run with `--manifest`.

use serde_json::{json, Value};

pub fn manifest() -> Value {
    json!({
        "id": "confluence",
        "name": "Confluence",
        "group": "Atlassian",
        "description": "Search and read self-hosted Confluence pages (Data Center / Server).",
        "binary": "confluence",
        "docs_url": "https://developer.atlassian.com/server/confluence/",
        "tools": [
            {"name": "confluence_search", "description": "Search Confluence pages with CQL."},
            {"name": "confluence_get_page", "description": "Read a Confluence page by ID."},
            {"name": "confluence_list_spaces", "description": "List accessible Confluence spaces."}
        ],
        "auth_fields": [
            {"env": "CONFLUENCE_URL", "label": "Confluence base URL", "kind": "text", "required": true, "help": "e.g. https://wiki.corp.com"},
            {"env": "CONFLUENCE_TOKEN", "label": "Personal Access Token", "kind": "secret", "required": false, "help": "Recommended for Data Center 7.9+. Generate in Profile, Personal Access Tokens."},
            {"env": "CONFLUENCE_USERNAME", "label": "Username", "kind": "text", "required": false, "help": "Use with password if you don't have a PAT."},
            {"env": "CONFLUENCE_PASSWORD", "label": "Password", "kind": "secret", "required": false}
        ],
        "advanced_fields": [
            {"env": "CONFLUENCE_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "CONFLUENCE_CA_BUNDLE", "label": "CA bundle path", "kind": "text", "required": false},
            {"env": "CONFLUENCE_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ],
        "notes": "Set CONFLUENCE_URL and either a token or username+password."
    })
}

pub fn print_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into())
    );
}
