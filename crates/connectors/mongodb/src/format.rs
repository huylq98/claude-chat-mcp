//! Plain-text rendering helpers for tool output.

use bson::Document;

/// Render a list of documents found by `mongo_find`. Each document is serialized
/// to JSON. The whole block is truncated to `max_len` characters.
pub fn format_documents(docs: &[Document], max_len: usize) -> String {
    if docs.is_empty() {
        return "No documents matched.".to_string();
    }
    let body = docs
        .iter()
        .map(|d| serde_json::to_string(d).unwrap_or_else(|e| format!("<unserializable: {e}>")))
        .collect::<Vec<_>>()
        .join("\n");
    let header = format!("{} document(s):\n", docs.len());
    connector_core::truncate(&format!("{header}{body}"), max_len)
}

/// Render a list of names (databases or collections), one per line.
pub fn format_names(label: &str, names: &[String]) -> String {
    if names.is_empty() {
        return format!("No {label}.");
    }
    let mut out = format!("{} {label}:\n", names.len());
    out.push_str(&names.join("\n"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_documents_empty() {
        assert_eq!(format_documents(&[], 1000), "No documents matched.");
    }

    #[test]
    fn format_names_empty() {
        assert_eq!(format_names("databases", &[]), "No databases.");
    }

    #[test]
    fn format_names_lists() {
        let names = vec!["a".to_string(), "b".to_string()];
        let out = format_names("collections", &names);
        assert!(out.starts_with("2 collections:"));
        assert!(out.contains("a"));
        assert!(out.contains("b"));
    }
}
