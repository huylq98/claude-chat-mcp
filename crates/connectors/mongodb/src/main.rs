//! Binary entrypoint for the `mongodb` connector.
//!
//! With `--manifest`, prints the connector manifest JSON and exits (used by the
//! configurator to discover this connector). Otherwise it connects to MongoDB
//! via the mongodb driver and boots the stdio MCP server.

mod config;
mod format;
mod handler;
mod manifest;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `--manifest`: print the connector descriptor and exit, before any tracing,
    // env reads, or connections (so the JSON is the only thing on stdout).
    if std::env::args().skip(1).any(|a| a == "--manifest") {
        manifest::print_manifest();
        return Ok(());
    }

    server_runtime::init_tracing();

    let server = handler::MongoServer::from_env().await?;

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

    tracing::info!("starting MongoDB MCP server");

    server_runtime::serve_stdio(server).await
}
