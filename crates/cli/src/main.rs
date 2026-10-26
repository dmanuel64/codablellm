mod cli;

use clap::Parser;
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
    // Init error handlers, parse args, and configure logger
    install_error_handlers()?;
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .with_writer(std::io::stderr)
        .init();

    Ok(())
}
