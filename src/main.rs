//! RustClaw - A high-performance Rust implementation of OpenClaw AI Agent Framework
//!
//! Features:
//! - Compatible with OpenClaw commands, plugins, and skills
//! - RPM (Requests Per Minute) control with random interval support
//! - Maximum concurrency control
//! - High-performance async runtime

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustclaw=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI arguments
    let cli = rustclaw_cli::Cli::parse_args();

    // Print banner
    rustclaw_cli::Cli::print_banner();

    // Run the command
    rustclaw_cli::commands::run(cli).await?;

    Ok(())
}
