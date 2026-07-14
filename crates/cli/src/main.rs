mod args;
mod config;
mod create;
mod errors;
mod repo;
mod resolver;

use std::io;

use clap::{ArgAction, Parser, Subcommand};
use codablellm::storage;
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
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    /// Number of times to greet
    #[arg(short, long, action = ArgAction::Count, default_value_t = config::get().display.console_log_level.into())]
    verbose: u8,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Create(CreateDatasetArgs),
    #[command(subcommand)]
    Repo(repo::Commands),
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
        tracing_appender::rolling::never(storage::STATE_DIR.clone(), "codablellm.log");
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

fn run() -> Result<()> {
    install_error_handlers()?;
    let args = Cli::parse();
    let _guard = init_logger(args.verbose);
    let no_input = !config::get().display.interactive;
    match args.command {
        Commands::Create(args) => {
            let dataset = create::create_dataset(resolver::resolve(args, no_input)?)?;
            Ok(())
        }
        Commands::Config(command) => config::run(command),
        Commands::Repo(commands) => todo!(),
    }
}

fn main() -> Result<()> {
    if let Err(report) = run() {
        if let Some(clap_err) = report.downcast_ref::<clap::Error>() {
            clap_err.exit()
        }
        Err(report)
    } else {
        Ok(())
    }
}
