use clap::{Args, Subcommand};
use codablellm::config;
use color_eyre::eyre::Result;

#[derive(Debug, Args)]
pub struct Command {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Get,
    Set,
    PrintPath,
}

pub fn run(command: Command) -> Result<()> {
    match command.command {
        Commands::PrintPath => println!("{}", config::PATH.display()),
        Commands::Get => todo!(),
        Commands::Set => todo!(),
    }
    Ok(())
}
