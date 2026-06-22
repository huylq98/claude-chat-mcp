mod client;
mod config;
mod format;
mod handler;
mod manifest;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `--manifest`: print the connector descriptor and exit, before any tracing
    // or stdio setup (so the JSON is the only thing on stdout).
    if std::env::args().any(|a| a == "--manifest") {
        manifest::print_manifest();
        return Ok(());
    }

    server_runtime::init_tracing();

    let server = handler::AirtableServer::from_env()?;

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

    tracing::info!(base_url = %server.base_url(), "starting Airtable MCP server");

    server_runtime::serve_stdio(server).await
}
