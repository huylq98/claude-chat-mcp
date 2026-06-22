//! Declarative description of this connector, consumed by the (future)
//! configurator wizard and the website registry. Emitted as JSON when the
//! binary is run with `--manifest`.

use serde_json::{json, Value};

pub fn manifest() -> Value {
    json!({
        "id": "redmine",
        "name": "Redmine",
        "group": "Productivity",
        "description": "Search and read self-hosted Redmine projects and issues (and create issues/notes in Writer mode).",
        "binary": "redmine",
        "docs_url": "https://www.redmine.org/projects/redmine/wiki/Rest_api",
        "tools": [
            {"name": "redmine_list_projects", "description": "List Redmine projects."},
            {"name": "redmine_list_issues", "description": "List issues in Redmine, optionally filtered by project."},
            {"name": "redmine_get_issue", "description": "Read a Redmine issue by its id."},
            {"name": "redmine_create_issue", "description": "Create a new issue in a Redmine project."},
            {"name": "redmine_add_note", "description": "Add a note to a Redmine issue."}
        ],
        "auth_fields": [
            {"env": "REDMINE_URL", "label": "Redmine base URL", "kind": "text", "required": true, "help": "e.g. https://redmine.corp.com"},
            {"env": "REDMINE_TOKEN", "label": "API access key", "kind": "secret", "required": true, "help": "Your Redmine API access key from My account > API access key"}
        ],
        "advanced_fields": [
            {"env": "REDMINE_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "REDMINE_CA_BUNDLE", "label": "CA bundle path", "kind": "text", "required": false},
            {"env": "REDMINE_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ],
        "notes": "Set REDMINE_URL and REDMINE_TOKEN. Write tools (create issue, add note) require Writer mode via REDMINE_MODE."
    })
}

pub fn print_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into())
    );
}
