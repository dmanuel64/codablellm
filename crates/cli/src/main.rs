mod args;
mod config;
mod create;
mod delete;
mod get;

use std::io;

use clap::{ArgAction, Parser, Subcommand};
use codablellm::{config::LogLevel, storage};
use color_eyre::eyre::Result;
use mimalloc::MiMalloc;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    /// Number of times to greet
    #[arg(short, long, action = ArgAction::Count, default_value_t = codablellm::config::get().display.console_log_level.into())]
    verbose: u8,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Create(create::Command),
    Config(config::Command),
}

fn install_error_handlers() -> Result<()> {
    color_eyre::install()?;
    human_panic::setup_panic!();
    Ok(())
}

pub fn init_logger(verbosity: u8) -> tracing_appender::non_blocking::WorkerGuard {
    let console_level: LogLevel = verbosity.into();
    let file_level = codablellm::config::get().display.file_log_level;

    let file_appender = tracing_appender::rolling::never(*storage::STATE_DIR, "codablellm.log");
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

fn main() -> Result<()> {
    install_error_handlers()?;
    let args = Cli::parse();
    let _guard = init_logger(args.verbose);
    match args.command {
        Commands::Create(command) => create::run(command),
        Commands::Config(command) => config::run(command),
    }
}
