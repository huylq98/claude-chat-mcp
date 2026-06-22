//! Read-only SQL guard plus markdown rendering of query results.
//!
//! The guard is intentionally conservative: it allows only a small allow-list of
//! statement leaders and rejects anything containing a write/DDL keyword as a
//! whole word. The error messages are written to be read by the calling model so
//! it can correct its query.

use crate::engine::{ColumnInfo, QueryResult};

/// Statement leaders we consider read-only.
const ALLOWED_LEADERS: &[&str] = &[
    "select", "show", "describe", "desc", "explain", "with", "pragma",
];

/// Keywords that, appearing as whole words anywhere in the statement, indicate a
/// write, DDL, or session-changing operation. `into` is included to block
/// `SELECT ... INTO ...` exfiltration/writes.
const FORBIDDEN_KEYWORDS: &[&str] = &[
    "insert", "update", "delete", "drop", "alter", "create", "truncate",
    "grant", "revoke", "replace", "merge", "call", "into", "set", "commit",
    "rollback",
];

/// Reject anything that is not a single read-only statement.
///
/// Returns `Err(message)` with a model-readable explanation when the query is
/// rejected; the caller surfaces that text directly to the model.
pub fn guard_read_only(sql: &str) -> Result<(), String> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err("Empty query. Provide a read-only SELECT/SHOW/DESCRIBE/EXPLAIN statement.".to_string());
    }

    // Reject multiple statements: a ';' anywhere other than as the final char
    // means more than one statement was supplied.
    let without_trailing = trimmed.strip_suffix(';').unwrap_or(trimmed);
    if without_trailing.contains(';') {
        return Err(
            "Multiple statements are not allowed. Submit a single read-only query without an internal ';'."
                .to_string(),
        );
    }

    let lower = without_trailing.to_lowercase();

    // Must start with an allowed leader (compared as a whole first word).
    let first_word: String = lower
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if !ALLOWED_LEADERS.contains(&first_word.as_str()) {
        return Err(format!(
            "Only read-only statements are allowed (must start with one of: {}). Got: '{}'.",
            ALLOWED_LEADERS.join(", "),
            first_word
        ));
    }

    // Reject forbidden keywords appearing as whole words.
    for kw in FORBIDDEN_KEYWORDS {
        if contains_word(&lower, kw) {
            return Err(format!(
                "The keyword '{kw}' is not permitted in read-only mode. Remove it and resubmit a SELECT/SHOW/DESCRIBE/EXPLAIN query."
            ));
        }
    }

    Ok(())
}

/// Whole-word containment check on already-lowercased text. A "word" boundary is
/// any character that is not alphanumeric or `_`.
fn contains_word(haystack: &str, word: &str) -> bool {
    let bytes = haystack.as_bytes();
    let wb = word.as_bytes();
    let mut i = 0;
    while let Some(pos) = find_from(haystack, word, i) {
        let before_ok = pos == 0 || !is_word_byte(bytes[pos - 1]);
        let after_idx = pos + wb.len();
        let after_ok = after_idx >= bytes.len() || !is_word_byte(bytes[after_idx]);
        if before_ok && after_ok {
            return true;
        }
        i = pos + 1;
    }
    false
}

fn find_from(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    haystack.get(from..)?.find(needle).map(|p| p + from)
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Render a [`QueryResult`] as a GitHub-flavoured markdown table.
pub fn format_rows(result: &QueryResult) -> String {
    if result.columns.is_empty() {
        return "(no columns returned)".to_string();
    }

    let mut out = String::new();
    // Header row.
    out.push_str("| ");
    out.push_str(&result.columns.iter().map(|c| escape_cell(c)).collect::<Vec<_>>().join(" | "));
    out.push_str(" |\n");
    // Separator row.
    out.push('|');
    for _ in &result.columns {
        out.push_str(" --- |");
    }
    out.push('\n');

    if result.rows.is_empty() {
        out.push_str("\n_(0 rows)_");
        return out;
    }

    // Data rows.
    for row in &result.rows {
        out.push_str("| ");
        out.push_str(&row.iter().map(|c| escape_cell(c)).collect::<Vec<_>>().join(" | "));
        out.push_str(" |\n");
    }

    out.push_str(&format!("\n_({} row(s))_", result.rows.len()));
    out
}

/// Render described columns as a markdown table.
pub fn format_columns(columns: &[ColumnInfo]) -> String {
    if columns.is_empty() {
        return "(no columns)".to_string();
    }
    let mut out = String::from("| Column | Type | Nullable | Key |\n| --- | --- | --- | --- |\n");
    for c in columns {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            escape_cell(&c.name),
            escape_cell(&c.data_type),
            if c.nullable { "YES" } else { "NO" },
            escape_cell(&c.key),
        ));
    }
    out
}

/// Render a list of names (databases / tables) as a markdown bullet list.
pub fn format_list(title: &str, items: &[String]) -> String {
    if items.is_empty() {
        return format!("No {title} found.");
    }
    let mut out = format!("{} ({}):\n\n", title, items.len());
    for item in items {
        out.push_str(&format!("- {item}\n"));
    }
    out
}

/// Make a cell safe for a markdown table: collapse newlines and escape pipes.
fn escape_cell(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\r', " ")
        .replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_select() {
        assert!(guard_read_only("SELECT * FROM t").is_ok());
        assert!(guard_read_only("  show tables ").is_ok());
        assert!(guard_read_only("WITH x AS (SELECT 1) SELECT * FROM x").is_ok());
        assert!(guard_read_only("SELECT 1;").is_ok()); // trailing ; allowed
    }

    #[test]
    fn rejects_writes() {
        assert!(guard_read_only("DELETE FROM t").is_err());
        assert!(guard_read_only("update t set a=1").is_err());
        assert!(guard_read_only("DROP TABLE t").is_err());
        assert!(guard_read_only("SELECT * INTO dump FROM t").is_err());
    }

    #[test]
    fn rejects_multiple_statements() {
        assert!(guard_read_only("SELECT 1; DROP TABLE t").is_err());
        assert!(guard_read_only("SELECT 1; SELECT 2").is_err());
    }

    #[test]
    fn word_boundary_does_not_flag_substrings() {
        // "created_at" / "settings" contain forbidden substrings but not as words.
        assert!(guard_read_only("SELECT created_at, settings FROM t").is_ok());
    }
}
