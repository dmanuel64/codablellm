mod create;
mod get;

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    pub verbose: u8,
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
        Commands::Create(command) => create::run(&command),
    }
}
