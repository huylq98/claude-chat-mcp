//! Markdown formatters - every tool returns an LLM-readable markdown string
//! rather than raw JSON. Jenkins' cryptic `color` codes are translated into a
//! plain human word so non-technical users understand a job's health.

use serde_json::Value;

fn s<'a>(v: &'a Value, ptr: &str) -> &'a str {
    v.pointer(ptr).and_then(Value::as_str).unwrap_or("")
}

/// Translate Jenkins' `color` field into a plain status word.
/// Jenkins encodes both status and an animated (building) variant in `color`,
/// e.g. `blue_anime` means "passing, currently building".
fn status_from_color(color: &str) -> String {
    match color {
        "blue" | "blue_anime" => "passing".to_string(),
        "red" | "red_anime" => "failing".to_string(),
        "yellow" | "yellow_anime" => "unstable".to_string(),
        "disabled" => "disabled".to_string(),
        "notbuilt" | "notbuilt_anime" => "notbuilt".to_string(),
        other => other.to_string(),
    }
}

pub fn jobs(data: &Value) -> String {
    let empty = vec![];
    let jobs = data
        .pointer("/jobs")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if jobs.is_empty() {
        return "No Jenkins jobs found.".into();
    }
    let mut out = vec![format!("## Jenkins jobs ({})\n", jobs.len())];
    for j in jobs {
        let name = s(j, "/name");
        let status = status_from_color(s(j, "/color"));
        out.push(format!("- **{name}** _(status: {status})_"));
    }
    out.join("\n")
}

pub fn job(data: &Value) -> String {
    let name = s(data, "/name");
    let url = s(data, "/url");
    let last_build = data
        .pointer("/lastBuild/number")
        .and_then(Value::as_u64)
        .map(|n| format!("#{n}"))
        .unwrap_or_else(|| "none".to_string());
    let health = data
        .pointer("/healthReport/0/description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut out = format!("# {name}\n- Last build: {last_build}");
    if !health.is_empty() {
        out.push_str(&format!("\n- Health: {health}"));
    }
    if !url.is_empty() {
        out.push_str(&format!("\n- URL: {url}"));
    }
    out
}

pub fn builds(data: &Value) -> String {
    let empty = vec![];
    let builds = data
        .pointer("/builds")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if builds.is_empty() {
        return "No builds found for this job.".into();
    }
    let mut out = vec![format!("## Builds ({})\n", builds.len())];
    for b in builds {
        let number = b.pointer("/number").and_then(Value::as_u64).unwrap_or(0);
        let result = b
            .pointer("/result")
            .and_then(Value::as_str)
            .unwrap_or("in progress");
        let duration = b
            .pointer("/duration")
            .and_then(Value::as_u64)
            .map(|ms| format!("{}s", ms / 1000))
            .unwrap_or_else(|| "?".to_string());
        out.push(format!(
            "- **#{number}** - {result} _(duration: {duration})_"
        ));
    }
    out.join("\n")
}

pub fn build(data: &Value) -> String {
    let number = data.pointer("/number").and_then(Value::as_u64).unwrap_or(0);
    let result = data
        .pointer("/result")
        .and_then(Value::as_str)
        .unwrap_or("in progress");
    let duration = data
        .pointer("/duration")
        .and_then(Value::as_u64)
        .map(|ms| format!("{}s", ms / 1000))
        .unwrap_or_else(|| "?".to_string());
    let timestamp = data
        .pointer("/timestamp")
        .and_then(Value::as_u64)
        .map(|ms| ms.to_string())
        .unwrap_or_else(|| "?".to_string());
    let url = s(data, "/url");
    let mut out = format!(
        "# Build #{number}\n- Result: {result}\n- Duration: {duration}\n- Started (epoch ms): {timestamp}"
    );
    if !url.is_empty() {
        out.push_str(&format!("\n- URL: {url}"));
    }
    out
}

pub fn triggered(job: &str) -> String {
    format!("Build triggered for **{job}**. It has been queued; check the builds list shortly for the new build.")
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
    fn jobs_renders_list_with_plain_status() {
        let data = json!({
            "jobs": [
                {"name": "build-app", "color": "blue", "url": "https://j/job/build-app/"},
                {"name": "deploy", "color": "red", "url": "https://j/job/deploy/"},
                {"name": "lint", "color": "yellow", "url": "https://j/job/lint/"}
            ]
        });
        let out = jobs(&data);
        assert!(out.contains("build-app"));
        assert!(out.contains("passing"));
        assert!(out.contains("failing"));
        assert!(out.contains("unstable"));
    }

    #[test]
    fn jobs_empty() {
        let data = json!({"jobs": []});
        assert_eq!(jobs(&data), "No Jenkins jobs found.");
    }
}
