//! The Mattermost MCP server: one stdio server exposing Mattermost tools.

use crate::client::MattermostClient;
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
pub struct ListChannelsArgs {
    /// The team id to list channels for.
    pub team_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPostsArgs {
    /// The channel id to read posts from.
    pub channel_id: String,
    /// Max posts to fetch (1-50, default 20).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PostMessageArgs {
    /// The channel id to post to.
    pub channel_id: String,
    /// The message text to post.
    pub message: String,
}

// ---- Server ------------------------------------------------------------------

#[derive(Clone)]
pub struct Server {
    client: Arc<MattermostClient>,
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
        let client = MattermostClient::from_config(&config)?;
        // Viewer mode strips the write tools so Claude only sees read-only ones.
        let mut tool_router = Self::tool_router();
        if connector_core::Mode::from_env("MATTERMOST_MODE").is_viewer() {
            for write_tool in ["mattermost_post_message"] {
                tool_router.remove_route(write_tool);
            }
        }
        Ok(Self {
            client: Arc::new(client),
            config: Arc::new(config),
            tool_router,
        })
    }

    fn max_len(&self) -> usize {
        self.config.max_content_length
    }

    #[tool(description = "List the Mattermost teams you are a member of.")]
    async fn mattermost_list_teams(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.list_teams().await {
            Ok(d) => format::teams(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "List channels in a Mattermost team.")]
    async fn mattermost_list_channels(
        &self,
        Parameters(args): Parameters<ListChannelsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.list_channels(&args.team_id).await {
            Ok(d) => format::channels(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Get recent posts in a Mattermost channel.")]
    async fn mattermost_get_posts(
        &self,
        Parameters(args): Parameters<GetPostsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = args.limit.unwrap_or(20).clamp(1, 50);
        let text = match self.client.get_posts(&args.channel_id, limit).await {
            Ok(d) => format::posts(&d, self.max_len()),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }

    #[tool(description = "Post a message to a Mattermost channel.")]
    async fn mattermost_post_message(
        &self,
        Parameters(args): Parameters<PostMessageArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let text = match self.client.create_post(&args.channel_id, &args.message).await {
            Ok(d) => format::created_post(&d),
            Err(e) => format!("Error ({}): {e}", e.status_code()),
        };
        ok(text)
    }
}

impl Server {
    /// Make one cheap authenticated call to verify the connection works.
    /// Used by the `--test-connection` binary mode.
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        self.client.current_user().await?;
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
                name: "mattermost".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Self-hosted Mattermost integration. Use the mattermost_* tools to list teams, \
                 list channels in a team, and read recent posts in a channel. In Writer mode you \
                 can also post a message to a channel. Use mattermost_list_teams to discover team \
                 ids and mattermost_list_channels to discover channel ids."
                    .to_string(),
            ),
        }
    }
}
