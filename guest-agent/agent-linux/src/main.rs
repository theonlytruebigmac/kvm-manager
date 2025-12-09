//! KVM Manager Guest Agent - Linux Implementation
//!
//! This agent runs inside Linux guest VMs and communicates with the host
//! via virtio-serial using JSON-RPC 2.0 protocol.

mod config;
mod handlers;
mod transport;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// KVM Manager Guest Agent for Linux
#[derive(Parser, Debug)]
#[command(name = "kvmmanager-agent")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Guest agent for KVM Manager")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "/etc/kvmmanager-agent/config.json")]
    config: PathBuf,

    /// Virtio-serial device path
    #[arg(short, long, default_value = "/dev/virtio-ports/org.kvmmanager.agent.0")]
    device: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Run in foreground (don't daemonize)
    #[arg(short, long)]
    foreground: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = args
        .log_level
        .parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("kvmmanager_agent_linux={}", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("KVM Manager Guest Agent v{} starting", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = match config::Config::load(&args.config) {
        Ok(cfg) => {
            info!("Loaded configuration from {:?}", args.config);
            cfg
        }
        Err(e) => {
            info!("Could not load config file: {}. Using defaults.", e);
            config::Config::default()
        }
    };

    // Validate virtio-serial device exists
    if !args.device.exists() {
        error!(
            "Virtio-serial device not found: {:?}. Ensure VM has virtio-serial channel configured.",
            args.device
        );
        anyhow::bail!("Virtio-serial device not found");
    }

    info!("Using virtio-serial device: {:?}", args.device);

    // Create transport
    let mut transport = transport::VirtioSerialTransport::new(&args.device, &config)
        .context("Failed to create virtio-serial transport")?;

    info!("Guest agent ready, listening for requests");

    // Main event loop
    loop {
        match transport.receive_request().await {
            Ok(request) => {
                info!("Received request: method={}", request.method);

                // Handle request
                let response = handlers::handle_request(request, &config).await;

                // Send response
                if let Err(e) = transport.send_response(response).await {
                    error!("Failed to send response: {}", e);
                }
            }
            Err(e) => {
                error!("Error receiving request: {}", e);

                // Try to reconnect after a delay
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                if let Err(e) = transport.reconnect().await {
                    error!("Failed to reconnect: {}", e);
                }
            }
        }
    }
}
