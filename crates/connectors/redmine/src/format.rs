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

pub fn projects(data: &Value) -> String {
    let empty = vec![];
    let projects = data
        .pointer("/projects")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if projects.is_empty() {
        return "No Redmine projects found.".into();
    }
    let mut out = vec![format!("## Redmine projects ({})\n", projects.len())];
    for p in projects {
        let id = n(p, "/id");
        let name = s(p, "/name");
        let identifier = s(p, "/identifier");
        let desc = s(p, "/description");
        if desc.is_empty() {
            out.push(format!("- **{name}** (id `{id}`, `{identifier}`)"));
        } else {
            out.push(format!("- **{name}** (id `{id}`, `{identifier}`) - {desc}"));
        }
    }
    out.join("\n")
}

pub fn issues(data: &Value) -> String {
    let empty = vec![];
    let issues = data
        .pointer("/issues")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if issues.is_empty() {
        return "No Redmine issues found.".into();
    }
    let mut out = vec![format!("## Redmine issues ({})\n", issues.len())];
    for i in issues {
        let id = n(i, "/id");
        let subject = s(i, "/subject");
        let status = i
            .pointer("/status/name")
            .and_then(Value::as_str)
            .unwrap_or("?");
        let priority = i
            .pointer("/priority/name")
            .and_then(Value::as_str)
            .unwrap_or("?");
        out.push(format!(
            "- **#{id}** - {subject} _(status: {status}, priority: {priority})_"
        ));
    }
    out.join("\n")
}

pub fn issue(data: &Value, max_len: usize) -> String {
    let issue = data.pointer("/issue").unwrap_or(data);
    let id = n(issue, "/id");
    let subject = s(issue, "/subject");
    let status = issue
        .pointer("/status/name")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let priority = issue
        .pointer("/priority/name")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let author = issue
        .pointer("/author/name")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let assignee = issue
        .pointer("/assigned_to/name")
        .and_then(Value::as_str)
        .unwrap_or("Unassigned");
    let description = s(issue, "/description");
    let desc = truncate(&strip_html(description), max_len);
    format!(
        "# #{id}: {subject}\n\
         - Status: {status}\n- Priority: {priority}\n- Author: {author}\n\
         - Assignee: {assignee}\n\n{desc}"
    )
}

pub fn created_issue(data: &Value) -> String {
    let issue = data.pointer("/issue").unwrap_or(data);
    let id = n(issue, "/id");
    let subject = s(issue, "/subject");
    format!("Created issue **#{id}**: {subject}")
}

pub fn added_note(issue_id: u64) -> String {
    format!("Added note to issue **#{issue_id}**.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn issues_renders_list() {
        let data = json!({
            "issues": [
                {"id": 1, "subject": "Fix bug",
                 "status": {"name": "New"}, "priority": {"name": "High"}}
            ]
        });
        let out = issues(&data);
        assert!(out.contains("#1"));
        assert!(out.contains("Fix bug"));
        assert!(out.contains("New"));
        assert!(out.contains("High"));
    }

    #[test]
    fn issues_empty() {
        let data = json!({ "issues": [] });
        assert_eq!(issues(&data), "No Redmine issues found.");
    }
}
