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

    if std::env::args().any(|a| a == "--test-connection") {
        match server.test_connection().await {
            Ok(()) => {
                println!("Connection OK");
                return Ok(());
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }

    tracing::info!("starting Elasticsearch MCP server");
    server_runtime::serve_stdio(server).await
}
