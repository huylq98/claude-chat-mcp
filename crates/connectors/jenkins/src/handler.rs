//! The Jenkins MCP server: one stdio server exposing Jenkins tools.

use crate::client::JenkinsClient;
use crate::config::Config;
use crate::format;
use rmcp::schemars::JsonSchema;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use std::sync::Arc;

// ---- Tool argument types -----------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListJobsArgs {
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetJobArgs {
    /// The job name (top-level), e.g. `build-app`.
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListBuildsArgs {
    /// The job name (top-level).
    pub job: String,
    /// Max results (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetBuildArgs {
    /// The job name (top-level).
    pub job: String,
    /// The build number.
    pub number: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TriggerBuildArgs {
    /// The job name (top-level) to trigger a build for.
    pub job: String,
}

// ---- Server ------------------------------------------------------------------

#[derive(Clone)]
pub struct Server {
    client: Arc<JenkinsClient>,
    #[allow(dead_code)]
    config: Arc<Config>,
    tool_router: ToolRouter<Server>,
}

fn ok(text: String) -> Result<CallToolResult, rmcp::ErrorData> {
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

#[tool_router]
impl Server {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = Config::from_env();
        config.validate()?;
        let client = JenkinsClient::from_config(&config)?;
        // Viewer mode strips the write tools so Claude only sees read-only ones.
        let mut tool_router = Self::tool_router();
        if connector_core::Mode::from_env("JENKINS_MODE").is_viewer() {
            tool_router.remove_route("jenkins_trigger_build");
        }
        Ok(Self {
            client: Arc::new(client),
            config: Arc::new(config),
            tool_router,
        })
    }

    #[tool(description = "List Jenkins jobs and their status.")]
    async fn jenkins_list_jobs(
        &self,
        Parameters(args): Parameters<ListJobsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let _limit = args.limit.unwrap_or(20).clamp(1, 50);
        let text = match self.client.list_jobs().await {
            Ok(d) => format::jobs(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Get a single Jenkins job by name.")]
    async fn jenkins_get_job(
        &self,
        Parameters(args): Parameters<GetJobArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.get_job(&args.name).await {
            Ok(d) => format::job(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "List recent builds for a Jenkins job.")]
    async fn jenkins_list_builds(
        &self,
        Parameters(args): Parameters<ListBuildsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let _limit = args.limit.unwrap_or(20).clamp(1, 50);
        let text = match self.client.list_builds(&args.job).await {
            Ok(d) => format::builds(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Get a single Jenkins build by job name and build number.")]
    async fn jenkins_get_build(
        &self,
        Parameters(args): Parameters<GetBuildArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.get_build(&args.job, args.number).await {
            Ok(d) => format::build(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Trigger a new build for a Jenkins job.")]
    async fn jenkins_trigger_build(
        &self,
        Parameters(args): Parameters<TriggerBuildArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.trigger_build(&args.job).await {
            Ok(_) => format::triggered(&args.job),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }
}

impl Server {
    /// Make one cheap authenticated call to verify the connection works.
    /// Used by the `--test-connection` binary mode.
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        self.client.root_info().await?;
        Ok(())
    }
}

#[tool_handler]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "jenkins".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Self-hosted Jenkins integration. Use the jenkins_* tools to list jobs, \
                 read a job, list its builds, and read a single build. In Writer mode you \
                 can also trigger a new build. Job names are top-level (folder jobs are not \
                 supported)."
                    .to_string(),
            ),
        }
    }
}
