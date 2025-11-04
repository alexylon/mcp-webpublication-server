mod models;
mod service;

use anyhow::Result;
use rmcp::ServiceExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use service::WebPublication;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcp_webpublication_server=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting MCP Webpublication server");

    let webpub = WebPublication::new()?;
    let server = webpub.serve(rmcp::transport::stdio()).await?;
    server.waiting().await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}
