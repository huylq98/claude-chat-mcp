//! Declarative description of this connector, consumed by the (future)
//! configurator wizard and the website registry. Emitted as JSON when the
//! binary is run with `--manifest`.

use serde_json::{json, Value};

pub fn manifest() -> Value {
    json!({
        "id": "jenkins",
        "name": "Jenkins",
        "group": "Dev",
        "description": "Browse self-hosted Jenkins jobs and builds (and trigger builds in Writer mode).",
        "binary": "jenkins",
        "docs_url": "https://www.jenkins.io/doc/book/using/remote-access-api/",
        "tools": [
            {"name": "jenkins_list_jobs", "description": "List Jenkins jobs and their status."},
            {"name": "jenkins_get_job", "description": "Get a single Jenkins job by name."},
            {"name": "jenkins_list_builds", "description": "List recent builds for a Jenkins job."},
            {"name": "jenkins_get_build", "description": "Get a single Jenkins build by job and number."},
            {"name": "jenkins_trigger_build", "description": "Trigger a new build for a Jenkins job."}
        ],
        "auth_fields": [
            {"env": "JENKINS_URL", "label": "Jenkins base URL", "kind": "text", "required": true, "help": "e.g. https://jenkins.corp.com"},
            {"env": "JENKINS_USER", "label": "Username", "kind": "text", "required": true},
            {"env": "JENKINS_TOKEN", "label": "API token", "kind": "secret", "required": true, "help": "Jenkins API token from your user > Configure > API Token"}
        ],
        "advanced_fields": [
            {"env": "JENKINS_PROXY_URL", "label": "Proxy URL", "kind": "text", "required": false},
            {"env": "JENKINS_CA_BUNDLE", "label": "CA bundle path", "kind": "text", "required": false},
            {"env": "JENKINS_SSL_VERIFY", "label": "Verify SSL", "kind": "bool", "required": false, "default": "true"}
        ],
        "notes": "Set JENKINS_URL, JENKINS_USER, and JENKINS_TOKEN (your Jenkins API token)."
    })
}

pub fn print_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into())
    );
}
