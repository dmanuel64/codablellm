// xtask/src/main.rs
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::Shell;
use std::{fs, io, path::Path};

#[path = "../../crates/cli/src/cli.rs"]
mod cli;

const BIN_NAME: &str = "codablellm";

#[derive(Clone, Copy, ValueEnum)]
enum Task {
    /// Generate the man page
    Man,
    /// Generate shell completions
    Completions,
}

#[derive(Parser)]
#[command(about = "Project automation tasks")]
struct Xtask {
    /// Tasks to run
    #[arg(required_unless_present = "all")]
    tasks: Vec<Task>,

    /// Run every task
    #[arg(long, conflicts_with = "tasks")]
    all: bool,
}

fn main() -> io::Result<()> {
    let args = Xtask::parse();
    let tasks = if args.all {
        Task::value_variants().to_vec()
    } else {
        args.tasks
    };

    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/dist");
    for task in tasks {
        match task {
            Task::Man => man(&dist.join("man"))?,
            Task::Completions => completions(&dist.join("completions"))?,
        }
    }
    Ok(())
}

fn man(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let path = dir.join(format!("{BIN_NAME}.1"));
    clap_mangen::Man::new(cli::Cli::command()).render(&mut fs::File::create(&path)?)?;
    println!("wrote {}", path.display());
    Ok(())
}

fn completions(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let mut cmd = cli::Cli::command();
    for &shell in Shell::value_variants() {
        let path = clap_complete::generate_to(shell, &mut cmd, BIN_NAME, dir)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
