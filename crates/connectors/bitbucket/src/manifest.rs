//! Declarative description of this connector, consumed by the (future)
//! configurator wizard and the website registry. Emitted as JSON when the
//! binary is run with `--manifest`.

use serde_json::{json, Value};

pub fn manifest() -> Value {
    json!({
        "id": "bitbucket",
        "name": "Bitbucket",
        "group": "Atlassian",
        "description": "Browse self-hosted Bitbucket repositories, pull requests, and commits (Server / Data Center).",
        "binary": "bitbucket",
        "docs_url": "https://developer.atlassian.com/server/bitbucket/rest/",
        "tools": [
            {"name": "bitbucket_list_repos", "description": "List repos in a Bitbucket project."},
            {"name": "bitbucket_list_pull_requests", "description": "List a repo's pull requests."},
            {"name": "bitbucket_get_commits", "description": "List a repo's recent commits."},
            {"name": "bitbucket_add_pr_comment", "description": "Add a comment to a Bitbucket pull request."}
        ],
        "auth_fields": [
            {"env": "BITBUCKET_URL", "label": "Bitbucket base URL", "kind": "text", "required": true, "help": "e.g. https://bitbucket.corp.com"},
            {"env": "BITBUCKET_TOKEN", "label": "Personal Access Token", "kind": "secret", "required": false, "help": "Recommended for Data Center 5.5+. Generate in Profile, Personal Access Tokens."},
            {"env": "BITBUCKET_USERNAME", "label": "Username", "kind": "text", "required": false, "help": "Use with password if you don't have a PAT."},
            {"env": "BITBUCKET_PASSWORD", "label": "Password", "kind": "secret", "required": false}
        ],
        "advanced_fields": [
            {"env": "BITBUCKET_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "BITBUCKET_CA_BUNDLE", "label": "CA bundle path", "kind": "text", "required": false},
            {"env": "BITBUCKET_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ],
        "notes": "Set BITBUCKET_URL and either a token or username+password. Write tools (add PR comment) require Writer mode via BITBUCKET_MODE."
    })
}

pub fn print_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into())
    );
}
