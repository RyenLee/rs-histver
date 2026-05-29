use anyhow::Result;
use clap::Parser;
use rs_histver::cli::Cli;
use rs_histver::ConfigBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut builder = ConfigBuilder::new();
    if let Some(data_dir) = cli.data_dir {
        builder = builder.data_dir(data_dir);
    }
    if let Some(db_file) = cli.db_file {
        builder = builder.db_file(db_file);
    }
    if let Some(table_prefix) = cli.table_prefix {
        builder = builder.table_prefix(table_prefix);
    }
    if let Some(timeout) = cli.timeout {
        builder = builder.timeout(timeout);
    }
    if let Some(max_concurrency) = cli.max_concurrency {
        builder = builder.max_concurrency(max_concurrency);
    }

    let cfg = builder.build()?;
    let app = rs_histver::app::App::init(cfg)?;

    app.execute(cli.command).await
}
