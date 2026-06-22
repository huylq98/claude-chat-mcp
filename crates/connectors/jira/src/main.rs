mod client;
mod config;
mod format;
mod handler;
mod manifest;

use handler::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `--manifest` prints the connector descriptor (used by the configurator /
    // website registry) and exits without starting the server.
    if std::env::args().any(|a| a == "--manifest") {
        manifest::print_manifest();
        return Ok(());
    }

    server_runtime::init_tracing();
    let server = Server::from_env()?;
    tracing::info!("starting Jira MCP server");
    server_runtime::serve_stdio(server).await
}
