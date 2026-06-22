use serde::{Deserialize, Serialize};

/// The connector catalog, embedded at compile time.
///
/// Path note: the spec asked for `include_str!("../../site/registry.json")`,
/// but relative to this source file that resolves to `crates/site/registry.json`
/// which does not exist. We embed the copy in `resources/` instead (the lead
/// controls that directory alongside the connector binaries). A current copy of
/// the repo-root `site/registry.json` lives at
/// `crates/control-panel/resources/registry.json`; keep it in sync.
const REGISTRY_JSON: &str = include_str!("../resources/registry.json");

/// One credential / option input on a connector.
///
/// Fields are snake_case to match registry.json exactly. Do NOT add
/// `rename_all` here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub env: String,
    pub label: String,
    /// One of: text | secret | select | bool.
    pub kind: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Only present for `kind == "select"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
}

/// One tool the connector exposes (display only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connector {
    pub id: String,
    pub name: String,
    pub group: String,
    pub description: String,
    pub binary: String,
    #[serde(default)]
    pub docs_url: String,
    #[serde(default)]
    pub auth_fields: Vec<Field>,
    #[serde(default)]
    pub advanced_fields: Vec<Field>,
    #[serde(default)]
    pub tools: Vec<Tool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Registry {
    #[allow(dead_code)]
    pub version: u32,
    pub connectors: Vec<Connector>,
}

/// Parse the embedded registry. Panics on a malformed embedded file, which can
/// only happen at build time, never at runtime in a shipped binary.
pub fn load() -> Vec<Connector> {
    let reg: Registry =
        serde_json::from_str(REGISTRY_JSON).expect("embedded registry.json is malformed");
    reg.connectors
}
