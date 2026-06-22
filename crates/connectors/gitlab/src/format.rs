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
    let projects = data.as_array().unwrap_or(&empty);
    if projects.is_empty() {
        return "No GitLab projects matched.".into();
    }
    let mut out = vec![format!("## GitLab projects ({})\n", projects.len())];
    for p in projects {
        let id = n(p, "/id");
        let name = s(p, "/path_with_namespace");
        let desc = s(p, "/description");
        if desc.is_empty() {
            out.push(format!("- **{name}** (id `{id}`)"));
        } else {
            out.push(format!("- **{name}** (id `{id}`) - {desc}"));
        }
    }
    out.join("\n")
}

pub fn issues(data: &Value) -> String {
    let empty = vec![];
    let issues = data.as_array().unwrap_or(&empty);
    if issues.is_empty() {
        return "No GitLab issues matched.".into();
    }
    let mut out = vec![format!("## GitLab issues ({})\n", issues.len())];
    for i in issues {
        let iid = n(i, "/iid");
        let title = s(i, "/title");
        let state = s(i, "/state");
        let author = i
            .pointer("/author/name")
            .and_then(Value::as_str)
            .unwrap_or("?");
        out.push(format!(
            "- **#{iid}** - {title} _(state: {state}, author: {author})_"
        ));
    }
    out.join("\n")
}

pub fn issue(data: &Value, max_len: usize) -> String {
    let iid = n(data, "/iid");
    let title = s(data, "/title");
    let state = s(data, "/state");
    let author = data
        .pointer("/author/name")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let assignee = data
        .pointer("/assignee/name")
        .and_then(Value::as_str)
        .unwrap_or("Unassigned");
    let web_url = s(data, "/web_url");
    let description = s(data, "/description");
    let desc = truncate(&strip_html(description), max_len);
    format!(
        "# #{iid}: {title}\n\
         - State: {state}\n- Author: {author}\n- Assignee: {assignee}\n\
         - URL: {web_url}\n\n{desc}"
    )
}

pub fn merge_requests(data: &Value) -> String {
    let empty = vec![];
    let mrs = data.as_array().unwrap_or(&empty);
    if mrs.is_empty() {
        return "No GitLab merge requests matched.".into();
    }
    let mut out = vec![format!("## GitLab merge requests ({})\n", mrs.len())];
    for m in mrs {
        let iid = n(m, "/iid");
        let title = s(m, "/title");
        let state = s(m, "/state");
        let author = m
            .pointer("/author/name")
            .and_then(Value::as_str)
            .unwrap_or("?");
        out.push(format!(
            "- **!{iid}** - {title} _(state: {state}, author: {author})_"
        ));
    }
    out.join("\n")
}

pub fn created_issue(data: &Value) -> String {
    let iid = n(data, "/iid");
    let title = s(data, "/title");
    let web_url = s(data, "/web_url");
    format!("Created issue **#{iid}**: {title}\n{web_url}")
}

pub fn created_note(data: &Value) -> String {
    let id = n(data, "/id");
    format!("Added comment (note id `{id}`).")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn issues_renders_list() {
        let data = json!([
            {"iid": 1, "title": "Fix bug", "state": "opened",
             "author": {"name": "Alice"}}
        ]);
        let out = issues(&data);
        assert!(out.contains("#1"));
        assert!(out.contains("Fix bug"));
        assert!(out.contains("Alice"));
    }

    #[test]
    fn issues_empty() {
        let data = json!([]);
        assert_eq!(issues(&data), "No GitLab issues matched.");
    }
}
