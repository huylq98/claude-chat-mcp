//! Render Airtable JSON API responses into human/LLM-readable markdown strings.

use connector_core::truncate;
use serde_json::Value;

/// Stringify a single Airtable field value compactly.
///
/// Airtable field values can be scalars, arrays (multi-select, linked records,
/// attachments) or nested objects. We keep arrays/objects as compact JSON so the
/// model still sees the structure without flooding the output.
fn field_value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        // Arrays/objects: compact JSON (e.g. linked record IDs, attachments).
        other => serde_json::to_string(other).unwrap_or_else(|_| other.to_string()),
    }
}

/// Format a single record's `fields` object into `key: value` lines.
fn format_record_fields(fields: &Value) -> String {
    let Some(obj) = fields.as_object() else {
        return String::new();
    };
    if obj.is_empty() {
        return "  (no fields)".to_string();
    }
    obj.iter()
        .map(|(k, v)| format!("  {k}: {}", field_value_to_string(v)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a single record (from `get_record` or one entry of `list_records`).
pub fn format_record(record: &Value) -> String {
    let id = record
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("(unknown)");
    let created = record.get("createdTime").and_then(Value::as_str);
    let empty = Value::Object(serde_json::Map::new());
    let fields = record.get("fields").unwrap_or(&empty);

    let mut out = format!("### Record {id}\n");
    if let Some(c) = created {
        out.push_str(&format!("Created: {c}\n"));
    }
    out.push_str(&format_record_fields(fields));
    out
}

/// Format the `list_records` response, truncating the whole block to
/// `max_content_length` characters so large tables don't blow the budget.
pub fn format_records(data: &Value, max_content_length: usize) -> String {
    let empty = vec![];
    let records = data
        .get("records")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    if records.is_empty() {
        return "## Airtable records (0)\n\nNo records found.".to_string();
    }

    let body = records
        .iter()
        .map(format_record)
        .collect::<Vec<_>>()
        .join("\n\n");

    let mut out = format!("## Airtable records ({})\n\n{body}", records.len());
    if let Some(offset) = data.get("offset").and_then(Value::as_str) {
        // Airtable paginates via an opaque offset token; surface it so callers
        // know more records exist.
        out.push_str(&format!(
            "\n\n_More records available (pagination offset: {offset})._"
        ));
    }
    truncate(&out, max_content_length)
}

/// Format the `list_bases` (Meta API) response.
pub fn format_bases(data: &Value) -> String {
    let empty = vec![];
    let bases = data
        .get("bases")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    if bases.is_empty() {
        return "## Airtable bases (0)\n\nNo bases accessible with this token.".to_string();
    }

    let lines = bases
        .iter()
        .map(|b| {
            let id = b.get("id").and_then(Value::as_str).unwrap_or("(unknown)");
            let name = b.get("name").and_then(Value::as_str).unwrap_or("(unnamed)");
            let permission = b
                .get("permissionLevel")
                .and_then(Value::as_str)
                .map(|p| format!(" — {p}"))
                .unwrap_or_default();
            format!("- **{name}** (`{id}`){permission}")
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("## Airtable bases ({})\n\n{lines}", bases.len())
}

/// Format the `list_tables` (Meta API) response for one base.
pub fn format_tables(data: &Value) -> String {
    let empty = vec![];
    let tables = data
        .get("tables")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    if tables.is_empty() {
        return "## Tables (0)\n\nNo tables in this base.".to_string();
    }

    let blocks = tables
        .iter()
        .map(|t| {
            let id = t.get("id").and_then(Value::as_str).unwrap_or("(unknown)");
            let name = t.get("name").and_then(Value::as_str).unwrap_or("(unnamed)");
            let primary_field_id = t.get("primaryFieldId").and_then(Value::as_str);

            let fields_empty = vec![];
            let fields = t
                .get("fields")
                .and_then(Value::as_array)
                .unwrap_or(&fields_empty);

            let primary_name = primary_field_id.and_then(|pid| {
                fields
                    .iter()
                    .find(|f| f.get("id").and_then(Value::as_str) == Some(pid))
                    .and_then(|f| f.get("name").and_then(Value::as_str))
            });

            let field_lines = fields
                .iter()
                .map(|f| {
                    let fname = f.get("name").and_then(Value::as_str).unwrap_or("(unnamed)");
                    let ftype = f.get("type").and_then(Value::as_str).unwrap_or("unknown");
                    format!("  - {fname} ({ftype})")
                })
                .collect::<Vec<_>>()
                .join("\n");

            let primary_line = primary_name
                .map(|p| format!("\nPrimary field: {p}"))
                .unwrap_or_default();

            format!("### {name} (`{id}`){primary_line}\nFields:\n{field_lines}")
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!("## Tables ({})\n\n{blocks}", tables.len())
}

/// Format a created/updated record into a short confirmation block.
pub fn format_mutation(action: &str, record: &Value) -> String {
    let id = record
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("(unknown)");
    format!("## {action} record `{id}`\n\n{}", format_record(record))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn format_records_empty() {
        let out = format_records(&json!({"records": []}), 50_000);
        assert!(out.contains("records (0)"));
    }

    #[test]
    fn format_records_renders_fields() {
        let data = json!({
            "records": [
                {"id": "rec1", "fields": {"Name": "Alice", "Age": 30}}
            ]
        });
        let out = format_records(&data, 50_000);
        assert!(out.contains("rec1"));
        assert!(out.contains("Name: Alice"));
        assert!(out.contains("Age: 30"));
    }

    #[test]
    fn format_bases_lists_ids() {
        let data = json!({"bases": [{"id": "app123", "name": "CRM", "permissionLevel": "create"}]});
        let out = format_bases(&data);
        assert!(out.contains("CRM"));
        assert!(out.contains("app123"));
    }
}
