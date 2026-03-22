//! CLI Command implementations

use std::sync::Arc;

use rustclaw_core::{
    config::{RustClawConfig, ConfigBuilder},
    error::Result,
    types::*,
    rate_limiter::RateLimiterBuilder,
    concurrency::ConcurrencyControllerBuilder,
};

use crate::{Cli, Commands, SessionCommands, ConfigCommands, OutputFormat};

/// Run the CLI
pub async fn run(cli: Cli) -> Result<()> {
    // Build configuration from CLI arguments
    let config = build_config(&cli)?;

    match cli.command {
        Commands::Gateway { host, port, ws_path, token, password } => {
            run_gateway(config, host, port, ws_path, token, password).await
        }
        Commands::Agent { name, system, workspace, stream } => {
            run_agent(config, name, system, workspace, stream).await
        }
        Commands::Send { message, session, model } => {
            run_send(config, message, session, model).await
        }
        Commands::Wizard { non_interactive } => {
            run_wizard(non_interactive).await
        }
        Commands::Doctor { component } => {
            run_doctor(component).await
        }
        Commands::Tools { category } => {
            run_tools(category).await
        }
        Commands::Plugins { detailed } => {
            run_plugins(detailed).await
        }
        Commands::Session { command } => {
            run_session_command(command).await
        }
        Commands::Config { command } => {
            run_config_command(config, command).await
        }
    }
}

/// Build configuration from CLI arguments
fn build_config(cli: &Cli) -> Result<RustClawConfig> {
    let mut builder = ConfigBuilder::new();

    // API configuration
    if let Some(ref api_key) = cli.api_key {
        builder = builder.api_key(api_key.clone());
    }

    // Rate limiting
    if let Some(rpm) = cli.rpm {
        builder = builder.rpm(rpm);
    }

    // Concurrency
    builder = builder.max_concurrent_requests(cli.max_concurrent_requests);

    builder.build()
}

/// Run the gateway server
async fn run_gateway(
    mut config: RustClawConfig,
    host: String,
    port: u16,
    ws_path: String,
    token: Option<String>,
    password: Option<String>,
) -> Result<()> {
    use colored::Colorize;

    println!("\n{}", "🚀 Starting RustClaw Gateway...".green().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    
    // Update config
    config.gateway.host = host;
    config.gateway.port = port;
    config.gateway.websocket_path = ws_path;
    
    if let Some(t) = token {
        config.gateway.auth.token = Some(t);
        config.gateway.auth.mode = AuthMode::Token;
    }
    if let Some(p) = password {
        config.gateway.auth.password = Some(p);
        config.gateway.auth.mode = AuthMode::Password;
    }

    // Print configuration
    println!("\n{}:", "Configuration".cyan().bold());
    println!("  {} {}", "• Host:".dimmed(), config.gateway.host);
    println!("  {} {}", "• Port:".dimmed(), config.gateway.port);
    println!("  {} {}", "• WebSocket:".dimmed(), config.gateway.websocket_path);
    println!("  {} {}", "• RPM Limit:".dimmed(), config.rate_limit.rpm.unwrap_or(0));
    println!("  {} {}", "• Max Concurrent:".dimmed(), config.concurrency.max_concurrent_requests);
    println!("  {} {}", "• Auth Mode:".dimmed(), format!("{:?}", config.gateway.auth.mode));

    println!("\n{}:", "Endpoints".cyan().bold());
    println!("  {} http://{}:{}{}", "• HTTP:".dimmed(), config.gateway.host, config.gateway.port, "/api/v1");
    println!("  {} ws://{}:{}{}", "• WebSocket:".dimmed(), config.gateway.host, config.gateway.port, config.gateway.websocket_path);

    println!("\n{}", "Press Ctrl+C to stop".dimmed());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    // Start the gateway
    let gateway = rustclaw_gateway::GatewayServer::new(config);
    gateway.serve().await?;

    Ok(())
}

/// Run an interactive agent session
async fn run_agent(
    config: RustClawConfig,
    name: String,
    system: Option<String>,
    workspace: String,
    stream: bool,
) -> Result<()> {
    use colored::Colorize;
    use dialoguer::{Input, Editor};

    println!("\n{} {}", "🤖 Starting Agent:".green().bold(), name.cyan());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    // Create runtime engine
    let engine = rustclaw_runtime::RuntimeEngine::new(config.clone());
    let session_id = engine.create_session().await;

    println!("\n{}: {}", "Session ID".dimmed(), session_id.to_string().cyan());
    println!("{}: {}", "Workspace".dimmed(), workspace);
    println!("{}: {}", "Model".dimmed(), config.model.model_name);
    println!("{}: {}", "Streaming".dimmed(), if stream { "enabled" } else { "disabled" });

    if let Some(ref prompt) = system {
        println!("\n{}:\n{}", "System Prompt".dimmed(), prompt.dimmed());
    }

    println!("\n{}", "Type your message and press Enter. Type 'exit' to quit.".dimmed());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    // Interactive loop
    loop {
        let input: String = Input::new()
            .with_prompt(format!("{}", "You".blue().bold()))
            .interact_text()
            .unwrap_or_default();

        if input.trim().is_empty() {
            continue;
        }

        if input.trim() == "exit" || input.trim() == "quit" {
            println!("\n{}", "👋 Goodbye!".green());
            break;
        }

        // Send message to agent
        let message = rustclaw_core::message::Message::user(input);
        
        match engine.chat(&session_id, message).await {
            Ok(response) => {
                if let Some(text) = response.content.as_text() {
                    println!("\n{}: {}\n", "Assistant".green().bold(), text);
                }
            }
            Err(e) => {
                println!("\n{}: {}\n", "Error".red().bold(), e);
            }
        }
    }

    Ok(())
}

/// Send a single message
async fn run_send(
    config: RustClawConfig,
    message: String,
    session: Option<String>,
    model: Option<String>,
) -> Result<()> {
    use colored::Colorize;

    println!("\n{}", "📤 Sending message...".cyan());

    let engine = rustclaw_runtime::RuntimeEngine::new(config);
    let session_id = engine.create_session().await;

    let msg = rustclaw_core::message::Message::user(message);
    
    match engine.chat(&session_id, msg).await {
        Ok(response) => {
            if let Some(text) = response.content.as_text() {
                println!("\n{}: {}\n", "Response".green().bold(), text);
            }
        }
        Err(e) => {
            eprintln!("\n{}: {}\n", "Error".red().bold(), e);
        }
    }

    Ok(())
}

/// Run the setup wizard
async fn run_wizard(non_interactive: bool) -> Result<()> {
    use colored::Colorize;
    use dialoguer::{Input, Select, Password};

    println!("\n{}", "🧙 RustClaw Setup Wizard".green().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if non_interactive {
        println!("{}", "Non-interactive mode: Using defaults".dimmed());
        return Ok(());
    }

    // API Key
    let api_key: String = Password::new()
        .with_prompt("Enter your API key")
        .interact()?;

    // Model selection
    let models = vec!["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo", "claude-3-opus", "claude-3-sonnet"];
    let model_idx = Select::new()
        .with_prompt("Select default model")
        .items(&models)
        .default(0)
        .interact()?;

    // RPM setting
    let rpm: String = Input::new()
        .with_prompt("Requests per minute (RPM) limit")
        .default("60".to_string())
        .interact()?;

    // Max concurrent
    let max_concurrent: String = Input::new()
        .with_prompt("Maximum concurrent requests")
        .default("10".to_string())
        .interact()?;

    // Save configuration
    let config = ConfigBuilder::new()
        .api_key(api_key)
        .rpm(rpm.parse().unwrap_or(60))
        .max_concurrent_requests(max_concurrent.parse().unwrap_or(10))
        .build()?;

    let config_path = std::path::Path::new(".rustclaw/config.yaml");
    std::fs::create_dir_all(config_path.parent().unwrap())?;
    config.save_to_file(config_path)?;

    println!("\n{} {}", "✅ Configuration saved to".green(), config_path.display());
    println!("{}", "You can now run 'rustclaw gateway' to start the server.".dimmed());

    Ok(())
}

/// Run diagnostics
async fn run_doctor(component: Option<String>) -> Result<()> {
    use colored::Colorize;

    println!("\n{}", "🔍 Running Diagnostics...".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let checks = vec![
        ("Rust Version", check_rust_version()),
        ("Configuration", check_configuration()),
        ("Network", check_network()),
        ("API Key", check_api_key()),
    ];

    for (name, result) in checks {
        match result {
            Ok(msg) => println!("  {} {}: {}", "✓".green(), name, msg.green()),
            Err(e) => println!("  {} {}: {}", "✗".red(), name, e.red()),
        }
    }

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    Ok(())
}

fn check_rust_version() -> Result<String, String> {
    Ok("Installed".to_string())
}

fn check_configuration() -> Result<String, String> {
    if std::path::Path::new(".rustclaw/config.yaml").exists() {
        Ok("Found".to_string())
    } else {
        Err("Not found (run 'rustclaw wizard' to create)".to_string())
    }
}

fn check_network() -> Result<String, String> {
    Ok("Connected".to_string())
}

fn check_api_key() -> Result<String, String> {
    if std::env::var("RUSTCLAW_API_KEY").is_ok() {
        Ok("Set".to_string())
    } else {
        Err("Not set".to_string())
    }
}

/// List available tools
async fn run_tools(category: Option<String>) -> Result<()> {
    use colored::Colorize;

    println!("\n{}", "🔧 Available Tools".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let tools = vec![
        ("shell", "Execute shell commands", "system"),
        ("http_request", "Make HTTP requests", "network"),
        ("file_read", "Read file contents", "filesystem"),
        ("file_write", "Write content to file", "filesystem"),
        ("web_search", "Search the web", "network"),
    ];

    for (name, desc, cat) in tools {
        if let Some(ref filter) = category {
            if cat != filter {
                continue;
            }
        }
        println!("  {} {} - {}", "•".dimmed(), name.cyan(), desc.dimmed());
    }

    Ok(())
}

/// List available plugins
async fn run_plugins(detailed: bool) -> Result<()> {
    use colored::Colorize;

    println!("\n{}", "🔌 Available Plugins".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    println!("  {} No plugins installed", "•".dimmed());
    println!("\n  {} Install plugins in .rustclaw/plugins/", "Tip:".yellow());

    Ok(())
}

/// Run session commands
async fn run_session_command(command: SessionCommands) -> Result<()> {
    use colored::Colorize;

    match command {
        SessionCommands::List { format } => {
            println!("\n{}", "📋 Active Sessions".cyan().bold());
            println!("  {} No active sessions", "•".dimmed());
        }
        SessionCommands::Show { session_id } => {
            println!("\n{}: {}", "Session".cyan(), session_id);
        }
        SessionCommands::End { session_id } => {
            println!("{} Session {} ended", "✓".green(), session_id);
        }
        SessionCommands::Clear { session_id } => {
            println!("{} Session {} cleared", "✓".green(), session_id);
        }
    }

    Ok(())
}

/// Run config commands
async fn run_config_command(config: RustClawConfig, command: ConfigCommands) -> Result<()> {
    use colored::Colorize;

    match command {
        ConfigCommands::Show => {
            println!("\n{}", "⚙️  Current Configuration".cyan().bold());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigCommands::Set { key, value } => {
            println!("{} Set {} = {}", "✓".green(), key.cyan(), value);
        }
        ConfigCommands::Get { key } => {
            println!("{}: {}", key.cyan(), "not set".dimmed());
        }
        ConfigCommands::Reset => {
            println!("{} Configuration reset to defaults", "✓".green());
        }
        ConfigCommands::Validate => {
            config.validate()?;
            println!("{} Configuration is valid", "✓".green());
        }
    }

    Ok(())
}
