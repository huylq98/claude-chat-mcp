//! Declarative description of this connector, consumed by the (future)
//! configurator wizard and the website registry. Emitted as JSON when the
//! binary is run with `--manifest`.

use serde_json::{json, Value};

pub fn manifest() -> Value {
    json!({
        "id": "sentry",
        "name": "Sentry",
        "group": "Dev",
        "description": "Browse self-hosted Sentry projects and issues (and resolve or ignore issues in Writer mode).",
        "binary": "sentry",
        "docs_url": "https://docs.sentry.io/api/",
        "tools": [
            {"name": "sentry_list_projects", "description": "List Sentry projects you can access."},
            {"name": "sentry_list_issues", "description": "List issues in a Sentry project."},
            {"name": "sentry_get_issue", "description": "Read a Sentry issue by its id."},
            {"name": "sentry_update_issue_status", "description": "Resolve, ignore, or unresolve a Sentry issue."}
        ],
        "auth_fields": [
            {"env": "SENTRY_URL", "label": "Sentry base URL", "kind": "text", "required": true, "help": "e.g. https://sentry.corp.com"},
            {"env": "SENTRY_TOKEN", "label": "Auth token", "kind": "secret", "required": true, "help": "A Sentry auth token with project:read (and event:write for Writer mode)"}
        ],
        "advanced_fields": [
            {"env": "SENTRY_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "SENTRY_CA_BUNDLE", "label": "CA bundle path", "kind": "text", "required": false},
            {"env": "SENTRY_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ],
        "notes": "Set SENTRY_URL and SENTRY_TOKEN. The write tool (update issue status) requires Writer mode via SENTRY_MODE."
    })
}

pub fn print_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into())
    );
}
