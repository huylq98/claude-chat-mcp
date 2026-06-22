//! Markdown formatters - every tool returns an LLM-readable markdown string
//! rather than raw JSON.

use connector_core::{strip_html, truncate};
use serde_json::Value;

fn s<'a>(v: &'a Value, ptr: &str) -> &'a str {
    v.pointer(ptr).and_then(Value::as_str).unwrap_or("")
}

pub fn teams(data: &Value) -> String {
    let empty = vec![];
    let teams = data.as_array().unwrap_or(&empty);
    if teams.is_empty() {
        return "No Mattermost teams found.".into();
    }
    let mut out = vec![format!("## Mattermost teams ({})\n", teams.len())];
    for t in teams {
        let id = s(t, "/id");
        let name = s(t, "/name");
        let display = s(t, "/display_name");
        out.push(format!("- **{display}** (`{name}`, id `{id}`)"));
    }
    out.join("\n")
}

fn channel_type_label(t: &str) -> &str {
    match t {
        "O" => "public",
        "P" => "private",
        "D" => "direct",
        other => other,
    }
}

pub fn channels(data: &Value) -> String {
    let empty = vec![];
    let channels = data.as_array().unwrap_or(&empty);
    if channels.is_empty() {
        return "No Mattermost channels found.".into();
    }
    let mut out = vec![format!("## Mattermost channels ({})\n", channels.len())];
    for c in channels {
        let id = s(c, "/id");
        let display = s(c, "/display_name");
        let kind = channel_type_label(s(c, "/type"));
        out.push(format!("- **{display}** _({kind})_ (id `{id}`)"));
    }
    out.join("\n")
}

pub fn posts(data: &Value, max_len: usize) -> String {
    let empty = vec![];
    let order = data
        .pointer("/order")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if order.is_empty() {
        return "No posts in this channel.".into();
    }
    let mut out = vec![format!("## Mattermost posts ({})\n", order.len())];
    for post_id in order {
        let pid = post_id.as_str().unwrap_or("");
        let post = data.pointer(&format!("/posts/{pid}"));
        let post = match post {
            Some(p) => p,
            None => continue,
        };
        let user_id = s(post, "/user_id");
        let short_user = user_id.get(..8).unwrap_or(user_id);
        let message = truncate(&strip_html(s(post, "/message")), max_len);
        out.push(format!("- _{short_user}_: {message}"));
    }
    out.join("\n")
}

pub fn created_post(data: &Value) -> String {
    let id = s(data, "/id");
    let message = s(data, "/message");
    format!("Posted message (id `{id}`): {message}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn teams_renders_list() {
        let data = json!([
            {"id": "abc", "name": "core", "display_name": "Core Team"}
        ]);
        let out = teams(&data);
        assert!(out.contains("Core Team"));
        assert!(out.contains("core"));
        assert!(out.contains("abc"));
    }

    #[test]
    fn teams_empty() {
        let data = json!([]);
        assert_eq!(teams(&data), "No Mattermost teams found.");
    }

    #[test]
    fn channels_renders_list() {
        let data = json!([
            {"id": "ch1", "name": "general", "display_name": "General", "type": "O"},
            {"id": "ch2", "name": "secret", "display_name": "Secret", "type": "P"}
        ]);
        let out = channels(&data);
        assert!(out.contains("General"));
        assert!(out.contains("public"));
        assert!(out.contains("Secret"));
        assert!(out.contains("private"));
    }

    #[test]
    fn channels_empty() {
        let data = json!([]);
        assert_eq!(channels(&data), "No Mattermost channels found.");
    }

    #[test]
    fn posts_renders_in_order() {
        let data = json!({
            "order": ["p2", "p1"],
            "posts": {
                "p1": {"id": "p1", "message": "hello", "user_id": "user1234abcd", "create_at": 1},
                "p2": {"id": "p2", "message": "world", "user_id": "user5678efgh", "create_at": 2}
            }
        });
        let out = posts(&data, 50_000);
        let world_at = out.find("world").unwrap();
        let hello_at = out.find("hello").unwrap();
        assert!(world_at < hello_at);
        assert!(out.contains("user1234"));
    }

    #[test]
    fn posts_empty() {
        let data = json!({"order": [], "posts": {}});
        assert_eq!(posts(&data, 50_000), "No posts in this channel.");
    }
}
