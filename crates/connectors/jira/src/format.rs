//! Markdown formatters - every tool returns an LLM-readable markdown string
//! rather than raw JSON.

use connector_core::{strip_html, truncate};
use serde_json::Value;

fn s<'a>(v: &'a Value, ptr: &str) -> &'a str {
    v.pointer(ptr).and_then(Value::as_str).unwrap_or("")
}

pub fn search(data: &Value) -> String {
    let empty = vec![];
    let issues = data
        .pointer("/issues")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if issues.is_empty() {
        return "No Jira issues matched.".into();
    }
    let mut out = vec![format!("## Jira issues ({})\n", issues.len())];
    for i in issues {
        let key = s(i, "/key");
        let summary = s(i, "/fields/summary");
        let status = s(i, "/fields/status/name");
        let assignee = i
            .pointer("/fields/assignee/displayName")
            .and_then(Value::as_str)
            .unwrap_or("Unassigned");
        out.push(format!(
            "- **{key}** - {summary} _(status: {status}, assignee: {assignee})_"
        ));
    }
    out.join("\n")
}

pub fn issue(data: &Value, max_len: usize) -> String {
    let key = s(data, "/key");
    let summary = s(data, "/fields/summary");
    let status = s(data, "/fields/status/name");
    let issuetype = s(data, "/fields/issuetype/name");
    let priority = s(data, "/fields/priority/name");
    let assignee = data
        .pointer("/fields/assignee/displayName")
        .and_then(Value::as_str)
        .unwrap_or("Unassigned");
    let reporter = data
        .pointer("/fields/reporter/displayName")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let description = data
        .pointer("/fields/description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let desc = truncate(&strip_html(description), max_len);
    format!(
        "# {key}: {summary}\n\
         - Type: {issuetype}\n- Status: {status}\n- Priority: {priority}\n\
         - Assignee: {assignee}\n- Reporter: {reporter}\n\n{desc}"
    )
}

pub fn projects(data: &Value) -> String {
    let empty = vec![];
    // /rest/api/2/project returns a bare array.
    let projects = data.as_array().unwrap_or(&empty);
    if projects.is_empty() {
        return "No Jira projects found.".into();
    }
    let mut out = vec![format!("## Jira projects ({})\n", projects.len())];
    for p in projects {
        out.push(format!("- **{}** - key `{}`", s(p, "/name"), s(p, "/key")));
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
    fn search_renders_issues() {
        let data = json!({
            "issues": [
                {"key": "ENG-1", "fields": {"summary": "Fix bug",
                 "status": {"name": "Open"},
                 "assignee": {"displayName": "Alice"}}}
            ]
        });
        let out = search(&data);
        assert!(out.contains("ENG-1"));
        assert!(out.contains("Fix bug"));
        assert!(out.contains("Alice"));
    }

    #[test]
    fn search_empty() {
        let data = json!({"issues": []});
        assert_eq!(search(&data), "No Jira issues matched.");
    }
}
