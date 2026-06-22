//! Markdown formatters - every tool returns an LLM-readable markdown string
//! rather than raw JSON.

use serde_json::Value;

fn s<'a>(v: &'a Value, ptr: &str) -> &'a str {
    v.pointer(ptr).and_then(Value::as_str).unwrap_or("")
}

pub fn repos(data: &Value) -> String {
    let empty = vec![];
    let values = data
        .pointer("/values")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if values.is_empty() {
        return "No repositories found.".into();
    }
    let mut out = vec![format!("## Bitbucket repositories ({})\n", values.len())];
    for r in values {
        let name = s(r, "/name");
        let slug = s(r, "/slug");
        let state = s(r, "/state");
        out.push(format!("- **{name}** - slug `{slug}` ({state})"));
    }
    out.join("\n")
}

pub fn pull_requests(data: &Value) -> String {
    let empty = vec![];
    let values = data
        .pointer("/values")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if values.is_empty() {
        return "No pull requests found.".into();
    }
    let mut out = vec![format!("## Pull requests ({})\n", values.len())];
    for pr in values {
        let id = pr.pointer("/id").and_then(Value::as_i64).unwrap_or(0);
        let title = s(pr, "/title");
        let state = s(pr, "/state");
        let author = s(pr, "/author/user/displayName");
        let from = s(pr, "/fromRef/displayId");
        let to = s(pr, "/toRef/displayId");
        out.push(format!(
            "- **#{id} {title}** - {state}, by {author} ({from} -> {to})"
        ));
    }
    out.join("\n")
}

pub fn commits(data: &Value) -> String {
    let empty = vec![];
    let values = data
        .pointer("/values")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if values.is_empty() {
        return "No commits found.".into();
    }
    let mut out = vec![format!("## Commits ({})\n", values.len())];
    for c in values {
        let id = s(c, "/displayId");
        let msg = s(c, "/message");
        let author = s(c, "/author/name");
        let first_line = msg.lines().next().unwrap_or("");
        out.push(format!("- `{id}` {first_line} - {author}"));
    }
    out.join("\n")
}

/// Shared error rendering: surface the message to the model as text.
pub fn error(e: &connector_core::CoreError) -> String {
    let code = e.status_code();
    if code > 0 {
        format!("Error (HTTP {code}): {e}")
    } else {
        format!("Error: {e}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn repos_renders_values() {
        let data = json!({
            "values": [
                {"name": "core-api", "slug": "core-api", "state": "AVAILABLE"}
            ]
        });
        let out = repos(&data);
        assert!(out.contains("core-api"));
        assert!(out.contains("AVAILABLE"));
    }

    #[test]
    fn repos_empty() {
        let data = json!({"values": []});
        assert_eq!(repos(&data), "No repositories found.");
    }
}
