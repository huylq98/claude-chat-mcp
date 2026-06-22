//! Declarative description of this connector, consumed by the (future)
//! configurator wizard and the website registry. Emitted as JSON when the
//! binary is run with `--manifest`.

use serde_json::{json, Value};

pub fn manifest() -> Value {
    json!({
        "id": "elasticsearch",
        "name": "Elasticsearch",
        "group": "Data",
        "description": "Search self-hosted Elasticsearch or OpenSearch indices and read mappings (and index documents in Writer mode).",
        "binary": "elasticsearch",
        "docs_url": "https://www.elastic.co/guide/en/elasticsearch/reference/current/rest-apis.html",
        "tools": [
            {"name": "es_list_indices", "description": "List indices in the cluster."},
            {"name": "es_get_mapping", "description": "Get the field mapping for an index."},
            {"name": "es_search", "description": "Search an index with an optional Lucene query string."},
            {"name": "es_index_document", "description": "Index a JSON document into an index."}
        ],
        "auth_fields": [
            {"env": "ES_URL", "label": "Elasticsearch base URL", "kind": "text", "required": true, "help": "e.g. https://your-es-host:9200"},
            {"env": "ES_USER", "label": "Username", "kind": "text", "required": false},
            {"env": "ES_PASSWORD", "label": "Password", "kind": "secret", "required": false}
        ],
        "advanced_fields": [
            {"env": "ES_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "ES_CA_BUNDLE", "label": "CA bundle path", "kind": "text", "required": false},
            {"env": "ES_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ],
        "notes": "Set ES_URL. Credentials (ES_USER, ES_PASSWORD) are optional for unauthenticated clusters. The write tool (index document) requires Writer mode via ES_MODE."
    })
}

pub fn print_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into())
    );
}
