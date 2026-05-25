use anyhow::Result;
use clap::Parser;
use rs_histver::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Parse CLI arguments
    let cli = Cli::parse();

    // 2. Initialize application with default config
    let cfg = rs_histver::Config::new();
    let app = rs_histver::app::App::init(cfg)?;

    // 3. Execute business logic
    app.execute(cli.command).await
}
