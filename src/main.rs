mod app;
mod cli;
mod domain;
mod infra;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Parse CLI arguments
    let cli = Cli::parse();

    // 2. Load configuration
    let config_path = infra::Config::resolve_path(cli.config.as_deref());
    let cfg = infra::Config::load(&config_path)?;

    // 3. Initialize application
    let app = app::App::init(cfg)?;

    // 4. Execute business logic
    app.execute(cli.command).await
}
