//! Markdown formatters - every tool returns an LLM-readable markdown string
//! rather than raw JSON.

use connector_core::truncate;
use serde_json::Value;

fn s<'a>(v: &'a Value, ptr: &str) -> &'a str {
    v.pointer(ptr).and_then(Value::as_str).unwrap_or("")
}

/// Render the `/_cat/indices` array.
pub fn indices(data: &Value) -> String {
    let empty = vec![];
    let rows = data.as_array().unwrap_or(&empty);
    if rows.is_empty() {
        return "No indices found.".into();
    }
    let mut out = vec![format!("## Indices ({})\n", rows.len())];
    for r in rows {
        let index = s(r, "/index");
        let health = s(r, "/health");
        let status = s(r, "/status");
        let docs = s(r, "/docs.count");
        let size = s(r, "/store.size");
        out.push(format!(
            "- **{index}** _(health: {health}, status: {status}, docs: {docs}, size: {size})_"
        ));
    }
    out.join("\n")
}

/// Render the top-level field names and types from a `/_mapping` response.
///
/// Shape: `{ <index>: { mappings: { properties: { <field>: { type: ... } } } } }`.
pub fn mapping(data: &Value) -> String {
    let obj = match data.as_object() {
        Some(o) if !o.is_empty() => o,
        _ => return "No mapping found.".into(),
    };
    let mut out = vec![];
    for (index, body) in obj {
        out.push(format!("## Mapping for **{index}**\n"));
        let props = body.pointer("/mappings/properties").and_then(Value::as_object);
        match props {
            Some(props) if !props.is_empty() => {
                for (field, def) in props {
                    let ty = def.pointer("/type").and_then(Value::as_str).unwrap_or("object");
                    out.push(format!("- **{field}**: {ty}"));
                }
            }
            _ => out.push("_(no fields)_".into()),
        }
    }
    out.join("\n")
}

/// Render a `/_search` response: total count plus each hit's id and a
/// truncated `_source` preview.
pub fn search_results(data: &Value, max_len: usize) -> String {
    let total = data
        .pointer("/hits/total/value")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let empty = vec![];
    let hits = data
        .pointer("/hits/hits")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if hits.is_empty() {
        return format!("No hits (total: {total}).");
    }
    let mut out = vec![format!("## Search results ({} shown, total: {total})\n", hits.len())];
    for h in hits {
        let id = s(h, "/_id");
        let source = h.pointer("/_source").cloned().unwrap_or(Value::Null);
        let preview = serde_json::to_string(&source).unwrap_or_else(|_| "{}".into());
        out.push(format!("- **{id}**: {}", truncate(&preview, max_len)));
    }
    out.join("\n")
}

/// Render an index-document response: `{ _id, result }`.
pub fn indexed_document(data: &Value) -> String {
    let id = s(data, "/_id");
    let result = s(data, "/result");
    format!("Indexed document **{id}** (result: {result}).")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn indices_renders_list() {
        let data = json!([
            {"index": "logs-2026", "health": "green", "status": "open",
             "docs.count": "42", "store.size": "1.2mb"}
        ]);
        let out = indices(&data);
        assert!(out.contains("logs-2026"));
        assert!(out.contains("green"));
        assert!(out.contains("42"));
    }

    #[test]
    fn indices_empty() {
        let data = json!([]);
        assert_eq!(indices(&data), "No indices found.");
    }
}
