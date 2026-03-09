use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use obsidian_mcp::client::ObsidianClient;
use obsidian_mcp::server::ObsidianServer;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::tower::{
    StreamableHttpServerConfig, StreamableHttpService,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Transport {
    Stdio,
    Http,
}

#[derive(Parser)]
#[command(
    name = "obsidian-mcp",
    about = "MCP server for Obsidian vault operations"
)]
struct Cli {
    /// Obsidian REST API URL
    #[arg(
        long,
        env = "OBSIDIAN_API_URL",
        default_value = "https://127.0.0.1:27124"
    )]
    api_url: String,

    /// Obsidian REST API key
    #[arg(long, env = "OBSIDIAN_API_KEY")]
    api_key: String,

    /// Transport mode
    #[arg(long, env = "MCP_TRANSPORT", default_value = "stdio")]
    transport: Transport,

    /// MCP server listen port
    #[arg(long, env = "MCP_PORT", default_value = "3000")]
    port: u16,

    /// MCP server listen host
    #[arg(long, env = "MCP_HOST", default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("obsidian_mcp=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    // Create Obsidian API client
    let client = Arc::new(ObsidianClient::new(cli.api_url.clone(), cli.api_key));

    // Verify connection to Obsidian
    tracing::info!("Connecting to Obsidian at {}...", cli.api_url);
    match client.server_info().await {
        Ok(info) => tracing::info!("Connected to Obsidian: {:?}", info),
        Err(e) => {
            tracing::error!("Failed to connect to Obsidian: {}", e);
            std::process::exit(1);
        }
    }

    // Set up MCP server
    let cancel_token = CancellationToken::new();
    let config = StreamableHttpServerConfig {
        stateful_mode: true,
        cancellation_token: cancel_token.clone(),
        ..Default::default()
    };

    let session_manager = Arc::new(LocalSessionManager::default());
    let client_clone = client.clone();
    let service = StreamableHttpService::new(
        move || Ok(ObsidianServer::new(client_clone.clone())),
        session_manager,
        config,
    );

    let app = axum::Router::new().nest_service("/mcp", service);

    let addr: SocketAddr = format!("{}:{}", cli.host, cli.port).parse()?;
    tracing::info!("MCP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("Shutting down...");
            cancel_token.cancel();
        })
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_default_transport_is_stdio() {
        let cli = Cli::try_parse_from(["obsidian-mcp", "--api-key", "test123"]).unwrap();
        assert_eq!(cli.transport, Transport::Stdio);
    }

    #[test]
    fn test_transport_http() {
        let cli =
            Cli::try_parse_from(["obsidian-mcp", "--api-key", "test123", "--transport", "http"])
                .unwrap();
        assert_eq!(cli.transport, Transport::Http);
    }

    #[test]
    fn test_transport_stdio_explicit() {
        let cli =
            Cli::try_parse_from(["obsidian-mcp", "--api-key", "test123", "--transport", "stdio"])
                .unwrap();
        assert_eq!(cli.transport, Transport::Stdio);
    }

    #[test]
    fn test_invalid_transport_rejected() {
        let result =
            Cli::try_parse_from(["obsidian-mcp", "--api-key", "test123", "--transport", "grpc"]);
        assert!(result.is_err());
    }
}
