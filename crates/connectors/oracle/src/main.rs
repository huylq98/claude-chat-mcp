//! Binary entrypoint for the `oracle` connector.
//!
//! With `--manifest`, prints the connector manifest JSON and exits (used by the
//! configurator to discover this connector). Otherwise it connects to Oracle via
//! db-core and boots the stdio MCP server.
//!
//! Oracle support links against Oracle Instant Client, so it is feature-gated:
//! the whole workspace builds without the client, and the binary refuses to run
//! until rebuilt with `--features oracle`.

mod manifest;

/// Default Oracle listener port.
#[cfg(feature = "oracle")]
const DEFAULT_PORT: u32 = 1521;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `--manifest`: emit the manifest and exit before touching env/connections.
    if std::env::args().skip(1).any(|a| a == "--manifest") {
        manifest::print_manifest();
        return Ok(());
    }

    server_runtime::init_tracing();

    #[cfg(feature = "oracle")]
    {
        use db_core::{make_server, DbConnConfig, OracleEngine};
        use std::sync::Arc;

        let config = DbConnConfig::from_env(DEFAULT_PORT);
        let engine = OracleEngine::connect(&config).await?;
        let server = make_server(Arc::new(engine), config, "oracle");

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

    #[cfg(not(feature = "oracle"))]
    {
        Err(anyhow::anyhow!(
            "Oracle support is not compiled in. Rebuild with --features oracle and install Oracle Instant Client."
        ))
    }
}
