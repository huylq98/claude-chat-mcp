//! Markdown formatters - every tool returns an LLM-readable markdown string
//! rather than raw JSON.

use connector_core::{strip_html, truncate};
use serde_json::Value;

fn s<'a>(v: &'a Value, ptr: &str) -> &'a str {
    v.pointer(ptr).and_then(Value::as_str).unwrap_or("")
}

fn n(v: &Value, ptr: &str) -> i64 {
    v.pointer(ptr).and_then(Value::as_i64).unwrap_or(0)
}

/// Sentry returns `count` as a string ("123"); fall back to numeric.
fn count(v: &Value, ptr: &str) -> String {
    if let Some(s) = v.pointer(ptr).and_then(Value::as_str) {
        return s.to_string();
    }
    n(v, ptr).to_string()
}

pub fn projects(data: &Value) -> String {
    let empty = vec![];
    let projects = data.as_array().unwrap_or(&empty);
    if projects.is_empty() {
        return "No Sentry projects found.".into();
    }
    let mut out = vec![format!("## Sentry projects ({})\n", projects.len())];
    for p in projects {
        let name = s(p, "/name");
        let slug = s(p, "/slug");
        let org = s(p, "/organization/slug");
        if org.is_empty() {
            out.push(format!("- **{name}** (slug `{slug}`)"));
        } else {
            out.push(format!("- **{name}** (slug `{slug}`, org `{org}`)"));
        }
    }
    out.join("\n")
}

pub fn issues(data: &Value) -> String {
    let empty = vec![];
    let issues = data.as_array().unwrap_or(&empty);
    if issues.is_empty() {
        return "No Sentry issues matched.".into();
    }
    let mut out = vec![format!("## Sentry issues ({})\n", issues.len())];
    for i in issues {
        let short_id = s(i, "/shortId");
        let title = s(i, "/title");
        let level = s(i, "/level");
        let status = s(i, "/status");
        let events = count(i, "/count");
        out.push(format!(
            "- **{short_id}** - {title} _(level: {level}, status: {status}, events: {events})_"
        ));
    }
    out.join("\n")
}

pub fn issue(data: &Value, max_len: usize) -> String {
    let short_id = s(data, "/shortId");
    let title = s(data, "/title");
    let culprit = s(data, "/culprit");
    let level = s(data, "/level");
    let status = s(data, "/status");
    let events = count(data, "/count");
    let first_seen = s(data, "/firstSeen");
    let last_seen = s(data, "/lastSeen");
    let body = truncate(&strip_html(culprit), max_len);
    format!(
        "# {short_id}: {title}\n\
         - Level: {level}\n- Status: {status}\n- Events: {events}\n\
         - First seen: {first_seen}\n- Last seen: {last_seen}\n\n{body}"
    )
}

pub fn updated_status(data: &Value) -> String {
    let short_id = s(data, "/shortId");
    let status = s(data, "/status");
    let label = if short_id.is_empty() { "issue" } else { short_id };
    format!("Updated **{label}** status to `{status}`.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn issues_renders_list() {
        let data = json!([
            {"id": "1", "shortId": "PROJ-1", "title": "TypeError: boom",
             "level": "error", "status": "unresolved", "count": "42"}
        ]);
        let out = issues(&data);
        assert!(out.contains("PROJ-1"));
        assert!(out.contains("TypeError: boom"));
        assert!(out.contains("error"));
        assert!(out.contains("42"));
    }

    #[test]
    fn issues_empty() {
        let data = json!([]);
        assert_eq!(issues(&data), "No Sentry issues matched.");
    }
}
