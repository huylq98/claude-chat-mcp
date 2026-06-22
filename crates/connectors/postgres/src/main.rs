//! Binary entrypoint for the `postgres` connector.
//!
//! With `--manifest`, prints the connector manifest JSON and exits (used by the
//! configurator to discover this connector). Otherwise it connects to
//! PostgreSQL via db-core and boots the stdio MCP server.

mod manifest;

use db_core::{make_server, DbConnConfig, PostgresEngine};
use std::sync::Arc;

/// Default PostgreSQL port.
const DEFAULT_PORT: u32 = 5432;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `--manifest`: emit the manifest and exit before touching env/connections.
    if std::env::args().skip(1).any(|a| a == "--manifest") {
        manifest::print_manifest();
        return Ok(());
    }

    server_runtime::init_tracing();

    let config = DbConnConfig::from_env(DEFAULT_PORT);
    let engine = PostgresEngine::connect(&config).await?;
    let server = make_server(Arc::new(engine), config, "postgres");

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

    server_runtime::serve_stdio(server).await
}
