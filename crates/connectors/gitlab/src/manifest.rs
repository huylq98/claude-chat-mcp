//! Declarative description of this connector, consumed by the (future)
//! configurator wizard and the website registry. Emitted as JSON when the
//! binary is run with `--manifest`.

use serde_json::{json, Value};

pub fn manifest() -> Value {
    json!({
        "id": "gitlab",
        "name": "GitLab",
        "group": "Dev",
        "description": "Search and read self-hosted GitLab projects, issues, and merge requests (and create issues/comments in Writer mode).",
        "binary": "gitlab",
        "docs_url": "https://docs.gitlab.com/ee/api/",
        "tools": [
            {"name": "gitlab_search_projects", "description": "Search GitLab projects by name or path."},
            {"name": "gitlab_list_issues", "description": "List issues in a GitLab project."},
            {"name": "gitlab_get_issue", "description": "Read a GitLab issue by its internal id (iid)."},
            {"name": "gitlab_list_merge_requests", "description": "List merge requests in a GitLab project."},
            {"name": "gitlab_create_issue", "description": "Create a new issue in a GitLab project."},
            {"name": "gitlab_comment_issue", "description": "Add a comment to a GitLab issue."}
        ],
        "auth_fields": [
            {"env": "GITLAB_URL", "label": "GitLab base URL", "kind": "text", "required": true, "help": "e.g. https://gitlab.corp.com"},
            {"env": "GITLAB_TOKEN", "label": "Personal Access Token", "kind": "secret", "required": true, "help": "Personal Access Token with api or read_api scope"}
        ],
        "advanced_fields": [
            {"env": "GITLAB_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "GITLAB_CA_BUNDLE", "label": "CA bundle path", "kind": "text", "required": false},
            {"env": "GITLAB_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ],
        "notes": "Set GITLAB_URL and GITLAB_TOKEN. Write tools (create issue, comment) require Writer mode via GITLAB_MODE."
    })
}

pub fn print_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into())
    );
}
