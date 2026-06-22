//! Markdown formatters - every tool returns an LLM-readable markdown string
//! rather than raw JSON.

use connector_core::{strip_html, truncate};
use serde_json::Value;

fn s<'a>(v: &'a Value, ptr: &str) -> &'a str {
    v.pointer(ptr).and_then(Value::as_str).unwrap_or("")
}

pub fn search(data: &Value, base_url: &str) -> String {
    let empty = vec![];
    let results = data
        .pointer("/results")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if results.is_empty() {
        return "No Confluence pages matched.".into();
    }
    let mut out = vec![format!("## Confluence results ({})\n", results.len())];
    for r in results {
        let title = s(r, "/title");
        let id = s(r, "/id");
        let space = s(r, "/space/key");
        let webui = s(r, "/_links/webui");
        let link = if webui.is_empty() {
            String::new()
        } else {
            format!(" - {base_url}{webui}")
        };
        out.push(format!("- **{title}** (id `{id}`, space `{space}`){link}"));
    }
    out.join("\n")
}

pub fn page(data: &Value, base_url: &str, max_len: usize) -> String {
    let title = s(data, "/title");
    let id = s(data, "/id");
    let space = s(data, "/space/key");
    let body = s(data, "/body/storage/value");
    let text = truncate(&strip_html(body), max_len);
    let webui = s(data, "/_links/webui");
    let link = if webui.is_empty() {
        String::new()
    } else {
        format!("\n{base_url}{webui}")
    };
    format!("# {title}\nid `{id}` (space `{space}`){link}\n\n{text}")
}

pub fn spaces(data: &Value) -> String {
    let empty = vec![];
    let results = data
        .pointer("/results")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if results.is_empty() {
        return "No spaces found.".into();
    }
    let mut out = vec![format!("## Confluence spaces ({})\n", results.len())];
    for sp in results {
        out.push(format!(
            "- **{}** - key `{}` ({})",
            s(sp, "/name"),
            s(sp, "/key"),
            s(sp, "/type")
        ));
    }
    out.join("\n")
}

pub fn created_page(data: &Value, base_url: &str) -> String {
    let title = s(data, "/title");
    let id = s(data, "/id");
    let webui = s(data, "/_links/webui");
    let link = if webui.is_empty() {
        String::new()
    } else {
        format!("\n{base_url}{webui}")
    };
    format!("Created page **{title}** (id `{id}`).{link}")
}

pub fn created_comment(data: &Value) -> String {
    let id = s(data, "/id");
    format!("Added comment (id `{id}`).")
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
    fn search_renders_results() {
        let data = json!({
            "results": [
                {"title": "Deploy Guide", "id": "123", "space": {"key": "ENG"},
                 "_links": {"webui": "/display/ENG/Deploy"}}
            ]
        });
        let out = search(&data, "https://wiki.corp.com");
        assert!(out.contains("Deploy Guide"));
        assert!(out.contains("id `123`"));
        assert!(out.contains("https://wiki.corp.com/display/ENG/Deploy"));
    }

    #[test]
    fn search_empty() {
        let data = json!({"results": []});
        assert_eq!(search(&data, ""), "No Confluence pages matched.");
    }
}
