mod args;
mod config;
mod create;
mod delete;
mod get;

use clap::{ArgAction, Parser, Subcommand};
use color_eyre::eyre::Result;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    /// Number of times to greet
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Create(create::Command),
}

fn install_error_handlers() -> Result<()> {
    color_eyre::install()?;
    human_panic::setup_panic!();
    Ok(())
}

pub fn init_logger(verbosity: u8) {
    let level = match verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    env_logger::Builder::new().filter_level(level).init();
}

fn main() -> Result<()> {
    install_error_handlers()?;
    let args = Cli::parse();
    init_logger(args.verbose);
    match args.command {
        Commands::Create(command) => create::run(command),
    }
}
