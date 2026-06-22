//! Markdown formatters - every tool returns an LLM-readable markdown string
//! rather than raw JSON.

use connector_core::truncate;
use serde_json::Value;

fn s<'a>(v: &'a Value, ptr: &str) -> &'a str {
    v.pointer(ptr).and_then(Value::as_str).unwrap_or("")
}

pub fn dashboards(data: &Value) -> String {
    let empty = vec![];
    let dashboards = data.as_array().unwrap_or(&empty);
    if dashboards.is_empty() {
        return "No Grafana dashboards matched.".into();
    }
    let mut out = vec![format!("## Grafana dashboards ({})\n", dashboards.len())];
    for d in dashboards {
        let uid = s(d, "/uid");
        let title = s(d, "/title");
        let folder = s(d, "/folderTitle");
        if folder.is_empty() {
            out.push(format!("- **{title}** (uid `{uid}`)"));
        } else {
            out.push(format!("- **{title}** (uid `{uid}`, folder: {folder})"));
        }
    }
    out.join("\n")
}

pub fn dashboard(data: &Value, max_len: usize) -> String {
    let title = s(data, "/dashboard/title");
    let uid = s(data, "/dashboard/uid");
    let folder = s(data, "/meta/folderTitle");
    let empty = vec![];
    let panels = data
        .pointer("/dashboard/panels")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    let mut out = vec![format!("# {title}")];
    if !uid.is_empty() {
        out.push(format!("- UID: `{uid}`"));
    }
    if !folder.is_empty() {
        out.push(format!("- Folder: {folder}"));
    }
    out.push(format!("- Panels: {}\n", panels.len()));

    if panels.is_empty() {
        out.push("No panels.".into());
    } else {
        for p in panels {
            let p_title = s(p, "/title");
            let p_type = s(p, "/type");
            let name = if p_title.is_empty() {
                "(untitled)"
            } else {
                p_title
            };
            out.push(format!("- {name} _(type: {p_type})_"));
        }
    }
    truncate(&out.join("\n"), max_len)
}

pub fn datasources(data: &Value) -> String {
    let empty = vec![];
    let sources = data.as_array().unwrap_or(&empty);
    if sources.is_empty() {
        return "No Grafana data sources found.".into();
    }
    let mut out = vec![format!("## Grafana data sources ({})\n", sources.len())];
    for d in sources {
        let name = s(d, "/name");
        let ds_type = s(d, "/type");
        out.push(format!("- **{name}** _(type: {ds_type})_"));
    }
    out.join("\n")
}

pub fn alerts(data: &Value) -> String {
    let empty = vec![];
    let rules = data.as_array().unwrap_or(&empty);
    if rules.is_empty() {
        return "No Grafana alert rules found.".into();
    }
    let mut out = vec![format!("## Grafana alert rules ({})\n", rules.len())];
    for r in rules {
        let title = s(r, "/title");
        let folder = s(r, "/folderUID");
        if folder.is_empty() {
            out.push(format!("- **{title}**"));
        } else {
            out.push(format!("- **{title}** _(folder: {folder})_"));
        }
    }
    out.join("\n")
}

pub fn created_annotation(data: &Value) -> String {
    let id = data.pointer("/id").and_then(Value::as_i64).unwrap_or(0);
    let message = s(data, "/message");
    if message.is_empty() {
        format!("Created annotation (id `{id}`).")
    } else {
        format!("Created annotation (id `{id}`): {message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dashboards_renders_list() {
        let data = json!([
            {"uid": "abc", "title": "Prod Overview", "folderTitle": "Production"}
        ]);
        let out = dashboards(&data);
        assert!(out.contains("Prod Overview"));
        assert!(out.contains("abc"));
        assert!(out.contains("Production"));
    }

    #[test]
    fn dashboards_empty() {
        let data = json!([]);
        assert_eq!(dashboards(&data), "No Grafana dashboards matched.");
    }
}
