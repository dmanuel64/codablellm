mod config;
mod create;
mod errors;
mod resolver;
mod storage;

use std::io;

use clap::{ArgAction, Parser, Subcommand};
use color_eyre::eyre::Result;
use mimalloc::MiMalloc;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::{
    config::{ConfigArgs, LogLevel},
    create::CreateDatasetArgs,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
    #[clap(flatten)]
    create: CreateDatasetArgs,
    /// Number of times to greet
    #[arg(short, long, action = ArgAction::Count, default_value_t = config::get().display.console_log_level.into())]
    verbose: u8,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Config(ConfigArgs),
}

fn install_error_handlers() -> Result<()> {
    color_eyre::install()?;
    human_panic::setup_panic!();
    Ok(())
}

fn init_logger(verbosity: u8) -> tracing_appender::non_blocking::WorkerGuard {
    let console_level: LogLevel = verbosity.into();
    let file_level = config::get().display.file_log_level;

    let file_appender =
        tracing_appender::rolling::never(storage::STATE_DIR.as_ref(), "codablellm.log");
    let (non_blocking_writer, guard) = tracing_appender::non_blocking(file_appender);
    let console_layer = fmt::layer()
        .with_writer(io::stderr)
        .with_filter(EnvFilter::new(console_level.to_string()));
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking_writer)
        .with_filter(EnvFilter::new(file_level.to_string()));
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();
    guard
}

async fn run() -> Result<()> {
    install_error_handlers()?;
    let args = Cli::parse();
    let _guard = init_logger(args.verbose);
    let no_input = !config::get().display.interactive;
    match args.command {
        Some(Commands::Config(command)) => config::run(command),
        None => {
            create::create_dataset(resolver::resolve(args.create, no_input)?).await?;
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(report) = run().await {
        if let Some(clap_err) = report.downcast_ref::<clap::Error>() {
            clap_err.exit()
        }
        Err(report)
    } else {
        Ok(())
    }
}
