use clap::{Args, Subcommand};
use codablellm::config;
use color_eyre::eyre::Result;

#[derive(Debug, Args)]
struct ConfigOpts {
    #[arg(long)]
    unset: bool,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Command {
    #[clap(subcommand)]
    command: Option<Commands>,
    #[arg(long)]
    show_all: bool,
    #[arg(long, conflicts_with = "show_all")]
    show_path: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(name = "display.progress")]
    DisplayProgress {
        #[arg(action = clap::ArgAction::Set)]
        enable: Option<bool>,
        #[clap(flatten)]
        common: ConfigOpts,
    },
    #[command(name = "display.console-log-level")]
    DisplayConsoleLogLevel {
        level: Option<config::LogLevel>,
        #[clap(flatten)]
        common: ConfigOpts,
    },
    #[command(name = "display.file-log-level")]
    DisplayFileLogLevel {
        level: Option<config::LogLevel>,
        #[clap(flatten)]
        common: ConfigOpts,
    },
    #[command(name = "forge.github-token")]
    ForgeGitHubToken {
        token: Option<String>,
        #[clap(flatten)]
        common: ConfigOpts,
    },
    #[command(name = "forge.gitlab-token")]
    ForgeGitLabToken {
        token: Option<String>,
        #[clap(flatten)]
        common: ConfigOpts,
    },
}

pub fn run(command: Command) -> Result<()> {
    if command.show_all {
        println!("{:#?}", config::get());
        return Ok(());
    } else if command.show_path {
        println!("{}", config::PATH.display());
        return Ok(());
    }
    match command.command {
        _ => todo!(),
    }
    Ok(())
}
