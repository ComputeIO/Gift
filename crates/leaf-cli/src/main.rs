use anyhow::Result;
use clap::Parser;
use leaf_cli::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI args once to check for --verbose flag
    let cli = Cli::parse();

    if cli.verbose {
        if let Err(e) = leaf_cli::logging::setup_logging_verbose(None) {
            eprintln!("Warning: Failed to initialize logging: {}", e);
        }
    } else if let Err(e) = leaf_cli::logging::setup_logging(None) {
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }

    // Now run the CLI logic
    let result = leaf_cli::cli::run_cli(cli).await;

    if leaf::otel::otlp::is_otlp_initialized() {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        leaf::otel::otlp::shutdown_otlp();
    }

    result
}
