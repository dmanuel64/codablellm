mod cli;
mod logger;

use clap::Parser;
use cliclack::{input, intro, outro};
use color_eyre::eyre::Result;
use mimalloc::MiMalloc;

use crate::cli::Cli;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn install_error_handlers() -> Result<()> {
    color_eyre::install()?;
    human_panic::setup_panic!();
    Ok(())
}

fn main() -> Result<()> {
    // Init error handlers and parse args
    install_error_handlers()?;
    let args = Cli::parse();

    if let Some(command) = args.command {
        match command {
            cli::Commands::Completion => todo!(),
        }
    }

    intro("Create Dataset")?;
    // Configure logger
    logger::init(args.verbosity.tracing_level_filter());
    // let number: String = input("What is the meaning of life?")
    //     .placeholder("Not sure")
    //     .validate(|input: &String| {
    //         if input.is_empty() {
    //             Err("Value is required!")
    //         } else {
    //             Ok(())
    //         }
    //     })
    //     .interact()?;
    codablellm_core::create_dataset(args.repo.unwrap());
    outro("You're all set!")?;
    Ok(())
}
