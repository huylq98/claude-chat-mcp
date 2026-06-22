//! Declarative description of this connector, consumed by the (future)
//! configurator wizard and the website registry. Emitted as JSON when the
//! binary is run with `--manifest`.

use serde_json::{json, Value};

pub fn manifest() -> Value {
    json!({
        "id": "grafana",
        "name": "Grafana",
        "group": "Dev",
        "description": "Browse self-hosted Grafana dashboards, data sources, and alerts (and add annotations in Writer mode).",
        "binary": "grafana",
        "docs_url": "https://grafana.com/docs/grafana/latest/developers/http_api/",
        "tools": [
            {"name": "grafana_search_dashboards", "description": "Search Grafana dashboards by title."},
            {"name": "grafana_get_dashboard", "description": "Read a Grafana dashboard by its uid."},
            {"name": "grafana_list_datasources", "description": "List configured Grafana data sources."},
            {"name": "grafana_list_alerts", "description": "List Grafana alert rules."},
            {"name": "grafana_create_annotation", "description": "Create a Grafana annotation."}
        ],
        "auth_fields": [
            {"env": "GRAFANA_URL", "label": "Grafana base URL", "kind": "text", "required": true, "help": "e.g. https://grafana.corp.com"},
            {"env": "GRAFANA_TOKEN", "label": "API token", "kind": "secret", "required": true, "help": "Grafana service account token or API key"}
        ],
        "advanced_fields": [
            {"env": "GRAFANA_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "GRAFANA_CA_BUNDLE", "label": "CA bundle path", "kind": "text", "required": false},
            {"env": "GRAFANA_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ],
        "notes": "Set GRAFANA_URL and GRAFANA_TOKEN. The write tool (create annotation) requires Writer mode via GRAFANA_MODE."
    })
}

pub fn print_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into())
    );
}
