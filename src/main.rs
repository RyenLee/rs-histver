use anyhow::Result;
use clap::Parser;
use rs_histver::app::App;
use rs_histver::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    App::execute(cli.command).await
}