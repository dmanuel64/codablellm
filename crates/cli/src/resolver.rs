use std::io::{IsTerminal, stdin};

use clap::Command;
use color_eyre::Result;

use crate::errors;

pub trait IntoResolved {
    type Resolved;

    fn into_resolved(self, interactive: bool) -> Result<Self::Resolved>;
}

pub fn resolve<Args: IntoResolved>(args: Args, no_input: bool) -> Result<Args::Resolved> {
    let interactive = !no_input && stdin().is_terminal();
    args.into_resolved(interactive)
}

pub fn require_or_prompt<T>(
    value: Option<T>,
    name: &str,
    interactive: bool,
    prompt: impl FnOnce() -> Result<T>,
) -> Result<T> {
    match value {
        Some(v) => Ok(v),
        None if interactive => prompt(),
        None => Err(errors::user_error(format!(
            "missing required argument <{name}> (running non-interactively)"
        ))
        .into()),
    }
}
