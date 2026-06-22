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

pub fn repos(data: &Value) -> String {
    let empty = vec![];
    // The search API wraps results in `items`.
    let repos = data
        .pointer("/items")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if repos.is_empty() {
        return "No GitHub repositories matched.".into();
    }
    let mut out = vec![format!("## GitHub repositories ({})\n", repos.len())];
    for r in repos {
        let name = s(r, "/full_name");
        let desc = s(r, "/description");
        if desc.is_empty() {
            out.push(format!("- **{name}**"));
        } else {
            out.push(format!("- **{name}** - {desc}"));
        }
    }
    out.join("\n")
}

pub fn issues(data: &Value) -> String {
    let empty = vec![];
    let issues = data.as_array().unwrap_or(&empty);
    if issues.is_empty() {
        return "No GitHub issues matched.".into();
    }
    let mut out = vec![format!("## GitHub issues ({})\n", issues.len())];
    for i in issues {
        let number = n(i, "/number");
        let title = s(i, "/title");
        let state = s(i, "/state");
        let author = i
            .pointer("/user/login")
            .and_then(Value::as_str)
            .unwrap_or("?");
        out.push(format!(
            "- **#{number}** - {title} _(state: {state}, author: {author})_"
        ));
    }
    out.join("\n")
}

pub fn issue(data: &Value, max_len: usize) -> String {
    let number = n(data, "/number");
    let title = s(data, "/title");
    let state = s(data, "/state");
    let author = data
        .pointer("/user/login")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let assignee = data
        .pointer("/assignee/login")
        .and_then(Value::as_str)
        .unwrap_or("Unassigned");
    let html_url = s(data, "/html_url");
    let body = s(data, "/body");
    let desc = truncate(&strip_html(body), max_len);
    format!(
        "# #{number}: {title}\n\
         - State: {state}\n- Author: {author}\n- Assignee: {assignee}\n\
         - URL: {html_url}\n\n{desc}"
    )
}

pub fn pull_requests(data: &Value) -> String {
    let empty = vec![];
    let prs = data.as_array().unwrap_or(&empty);
    if prs.is_empty() {
        return "No GitHub pull requests matched.".into();
    }
    let mut out = vec![format!("## GitHub pull requests ({})\n", prs.len())];
    for p in prs {
        let number = n(p, "/number");
        let title = s(p, "/title");
        let state = s(p, "/state");
        let author = p
            .pointer("/user/login")
            .and_then(Value::as_str)
            .unwrap_or("?");
        out.push(format!(
            "- **#{number}** - {title} _(state: {state}, author: {author})_"
        ));
    }
    out.join("\n")
}

pub fn created_issue(data: &Value) -> String {
    let number = n(data, "/number");
    let title = s(data, "/title");
    let html_url = s(data, "/html_url");
    format!("Created issue **#{number}**: {title}\n{html_url}")
}

pub fn created_comment(data: &Value) -> String {
    let id = n(data, "/id");
    format!("Added comment (id `{id}`).")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn issues_renders_list() {
        let data = json!([
            {"number": 1, "title": "Fix bug", "state": "open",
             "user": {"login": "alice"}}
        ]);
        let out = issues(&data);
        assert!(out.contains("#1"));
        assert!(out.contains("Fix bug"));
        assert!(out.contains("alice"));
    }

    #[test]
    fn issues_empty() {
        let data = json!([]);
        assert_eq!(issues(&data), "No GitHub issues matched.");
    }
}
