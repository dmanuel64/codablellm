mod cli;
mod logger;

use std::path::{MAIN_SEPARATOR, Path, is_separator};

use clap::Parser;
use cliclack::{confirm, input, intro, multiselect, outro};
use color_eyre::eyre::Result;
use mimalloc::MiMalloc;
use strum::IntoEnumIterator;

use crate::cli::{Cli, SourceLanguage};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn install_error_handlers() -> Result<()> {
    color_eyre::install()?;
    human_panic::setup_panic!();
    Ok(())
}

fn suggest_dirs(query: &str) -> Vec<String> {
    let (dir, prefix) = match query.rfind(is_separator) {
        Some(i) => (&query[..=i], &query[i + 1..]),
        None => ("", query),
    };
    let Ok(entries) = std::fs::read_dir(if dir.is_empty() { "." } else { dir }) else {
        return Vec::new();
    };
    let mut dirs: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name.starts_with(prefix))
        .map(|name| format!("{dir}{name}{MAIN_SEPARATOR}"))
        .collect();
    dirs.sort();
    dirs
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

    // Start wizard
    intro("Create Dataset")?;
    // Configure logger
    logger::init(args.verbosity.tracing_level_filter());
    // Get optional args if they were not specified
    let repo = if let Some(repo) = args.repo {
        repo
    } else {
        input("Enter the path to the repository to create the dataset from")
            .validate(|input: &String| {
                if Path::new(input).is_dir() {
                    Ok(())
                } else {
                    Err("Not a directory")
                }
            })
            .autocomplete(suggest_dirs)
            .interact()?
    };
    let source_languages = if let Some(source_languages) = args.source_languages {
        source_languages
    } else {
        if !confirm("Automatically detect repository source code languages?")
            .initial_value(true)
            .interact()?
        {
            let items: Vec<_> = SourceLanguage::iter()
                .filter(|l| *l != SourceLanguage::Auto)
                .map(|l| (l, l.to_string(), ""))
                .collect();
            multiselect("Select the languages to extract")
                .items(&items)
                .interact()?
        } else {
            vec![SourceLanguage::Auto]
        }
    };
    // Create dataset
    // codablellm_core::create_dataset(repo);
    outro("You're all set!")?;
    Ok(())
}
