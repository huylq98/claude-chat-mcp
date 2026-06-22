//! Declarative description of this connector, consumed by the (future)
//! configurator wizard and the website registry. Emitted as JSON when the
//! binary is run with `--manifest`.

use serde_json::{json, Value};

pub fn manifest() -> Value {
    json!({
        "id": "jira",
        "name": "Jira",
        "group": "Atlassian",
        "description": "Search and read self-hosted Jira issues (Data Center / Server).",
        "binary": "jira",
        "docs_url": "https://developer.atlassian.com/server/jira/platform/rest/",
        "tools": [
            {"name": "jira_search", "description": "Search Jira issues with JQL."},
            {"name": "jira_get_issue", "description": "Read a Jira issue by key."},
            {"name": "jira_list_projects", "description": "List Jira projects."}
        ],
        "auth_fields": [
            {"env": "JIRA_URL", "label": "Jira base URL", "kind": "text", "required": true, "help": "e.g. https://jira.corp.com"},
            {"env": "JIRA_TOKEN", "label": "Personal Access Token", "kind": "secret", "required": false, "help": "Recommended for Data Center 7.9+. Generate in Profile, Personal Access Tokens."},
            {"env": "JIRA_USERNAME", "label": "Username", "kind": "text", "required": false, "help": "Use with password if you don't have a PAT."},
            {"env": "JIRA_PASSWORD", "label": "Password", "kind": "secret", "required": false}
        ],
        "advanced_fields": [
            {"env": "JIRA_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "JIRA_CA_BUNDLE", "label": "CA bundle path", "kind": "text", "required": false},
            {"env": "JIRA_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ],
        "notes": "Set JIRA_URL and either a token or username+password."
    })
}

pub fn print_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into())
    );
}
