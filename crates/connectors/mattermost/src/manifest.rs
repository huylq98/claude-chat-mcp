//! Declarative description of this connector, consumed by the (future)
//! configurator wizard and the website registry. Emitted as JSON when the
//! binary is run with `--manifest`.

use serde_json::{json, Value};

pub fn manifest() -> Value {
    json!({
        "id": "mattermost",
        "name": "Mattermost",
        "group": "Productivity",
        "description": "Read self-hosted Mattermost teams, channels, and messages (and post messages in Writer mode).",
        "binary": "mattermost",
        "docs_url": "https://api.mattermost.com/",
        "tools": [
            {"name": "mattermost_list_teams", "description": "List the Mattermost teams you are a member of."},
            {"name": "mattermost_list_channels", "description": "List channels in a Mattermost team."},
            {"name": "mattermost_get_posts", "description": "Get recent posts in a Mattermost channel."},
            {"name": "mattermost_post_message", "description": "Post a message to a Mattermost channel."}
        ],
        "auth_fields": [
            {"env": "MATTERMOST_URL", "label": "Mattermost base URL", "kind": "text", "required": true, "help": "e.g. https://chat.corp.com"},
            {"env": "MATTERMOST_TOKEN", "label": "Personal Access Token", "kind": "secret", "required": true, "help": "A personal access token from your Mattermost account settings"}
        ],
        "advanced_fields": [
            {"env": "MATTERMOST_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "MATTERMOST_CA_BUNDLE", "label": "CA bundle path", "kind": "text", "required": false},
            {"env": "MATTERMOST_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ],
        "notes": "Set MATTERMOST_URL and MATTERMOST_TOKEN. The write tool (post message) requires Writer mode via MATTERMOST_MODE."
    })
}

pub fn print_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into())
    );
}
