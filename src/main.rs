#[cfg(feature = "cli")]
use anyhow::Result;
#[cfg(feature = "cli")]
use clap::Parser;
#[cfg(feature = "cli")]
use rs_histver::app::App;
#[cfg(feature = "cli")]
use rs_histver::cli::Cli;

#[cfg(feature = "cli")]
fn main() -> Result<()> {
    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(App::execute(cli.command))
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("This binary requires the 'cli' feature to be enabled.");
    eprintln!("Use 'cargo run --features cli' or install rs-histver with CLI support.");
    std::process::exit(1);
}
