use color_eyre::eyre::Result;

#[derive(Debug, Args)]
pub struct Command {
    #[clap(subcommand)]
    command: Commands,
    /// Print the path to the config file and exit
    #[arg(long, exclusive = true)]
    pub path: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Get,
    Set,
}

pub fn run(command: Command) -> Result<()> {
    if command.path {
        println!("{}", config::path().display());
        return Ok(());
    }
    Ok(())
}
