//! Declarative description of this connector, consumed by the (future)
//! configurator wizard and the website registry. Emitted as JSON when the
//! binary is run with `--manifest`.

use serde_json::{json, Value};

pub fn manifest() -> Value {
    json!({
        "id": "github",
        "name": "GitHub",
        "group": "Dev",
        "description": "Search and read self-hosted GitHub repositories, issues, and pull requests (and create issues/comments in Writer mode).",
        "binary": "github",
        "docs_url": "https://docs.github.com/en/enterprise-server/rest",
        "tools": [
            {"name": "github_search_repos", "description": "Search GitHub repositories by name or metadata."},
            {"name": "github_list_issues", "description": "List issues in a GitHub repository."},
            {"name": "github_get_issue", "description": "Read a GitHub issue by its number."},
            {"name": "github_list_pull_requests", "description": "List pull requests in a GitHub repository."},
            {"name": "github_create_issue", "description": "Create a new issue in a GitHub repository."},
            {"name": "github_comment_issue", "description": "Add a comment to a GitHub issue."}
        ],
        "auth_fields": [
            {"env": "GITHUB_URL", "label": "GitHub API URL", "kind": "text", "required": true, "help": "Your GitHub Enterprise API URL, e.g. https://github.company.com/api/v3"},
            {"env": "GITHUB_TOKEN", "label": "Personal Access Token", "kind": "secret", "required": true, "help": "Personal Access Token with repo or read-only scope"}
        ],
        "advanced_fields": [
            {"env": "GITHUB_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "GITHUB_CA_BUNDLE", "label": "CA bundle path", "kind": "text", "required": false},
            {"env": "GITHUB_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ],
        "notes": "Set GITHUB_URL and GITHUB_TOKEN. Write tools (create issue, comment) require Writer mode via GITHUB_MODE."
    })
}

pub fn print_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into())
    );
}
