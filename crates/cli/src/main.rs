use mimalloc::MiMalloc;
mod create;
mod get;

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
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

fn main() -> Result<()> {
    install_error_handlers()?;
    let args = Cli::parse();
    match args.command {
        Commands::Create(command) => create::run(command),
    }
}
