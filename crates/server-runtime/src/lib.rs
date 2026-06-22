//! Thin shared host for connector MCP servers.
//!
//! Every connector binary is a stdio MCP server launched by Claude Desktop. The
//! per-connector crate defines its tools (via rmcp's `#[tool_router]`); this
//! crate owns the boilerplate they all share: stderr-only logging (stdout is
//! reserved for the MCP wire protocol) and the stdio serve loop.

use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

// Re-export so connectors can build the transport directly if they need to.
pub use rmcp::{transport::stdio as stdio_transport, ServiceExt as _ServiceExt};

/// Initialise tracing to stderr. NEVER log to stdout in stdio mode — stdout is
/// the MCP JSON-RPC channel and any stray bytes corrupt the protocol.
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
}

/// Serve a connector's rmcp handler over stdio until the client disconnects.
pub async fn serve_stdio<S>(handler: S) -> Result<()>
where
    S: ServerHandlerLike,
{
    let service = handler.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

/// Bound alias: anything rmcp accepts as a server handler over a transport.
pub trait ServerHandlerLike: rmcp::ServerHandler {}
impl<T: rmcp::ServerHandler> ServerHandlerLike for T {}
