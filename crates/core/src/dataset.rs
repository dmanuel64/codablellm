use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use polars::prelude::*;
use thiserror::Error;

use crate::{FileSource, function::Function, storage};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create dataset: {0}")]
    DatasetCreation(#[source] PolarsError),
    #[error("script failure")]
    ScriptError,
}

#[derive(Debug, Default)]
pub struct Options {
    pub display_progress: bool,
}

pub trait Dataset {
    fn df(&self) -> &DataFrame;
    fn df_mut(&mut self) -> &mut DataFrame;
}

pub enum Kind {
    Source(SourceDataset),
    Binary(BinaryDataset),
}

impl Dataset for Kind {
    fn df(&self) -> &DataFrame {
        match self {
            Self::Source(d) => d.df(),
            Self::Binary(d) => d.df(),
        }
    }

    fn df_mut(&mut self) -> &mut DataFrame {
        match self {
            Self::Source(d) => d.df_mut(),
            Self::Binary(d) => d.df_mut(),
        }
    }
}

pub struct SourceDataset {
    df: DataFrame,
}

impl Dataset for SourceDataset {
    fn df(&self) -> &DataFrame {
        &self.df
    }

    fn df_mut(&mut self) -> &mut DataFrame {
        &mut self.df
    }
}

impl SourceDataset {
    pub fn new(functions: &Vec<Function>) -> Result<Self, Error> {
        let names: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        let definitions: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        let df = df!(
            "name" => names,
            "definitions" => definitions,
        )
        .map_err(|e| Error::DatasetCreation(e))?;
        Ok(Self { df })
    }
}

pub struct BinaryDataset {
    df: DataFrame,
}

impl Dataset for BinaryDataset {
    fn df(&self) -> &DataFrame {
        &self.df
    }

    fn df_mut(&mut self) -> &mut DataFrame {
        &mut self.df
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "value-enums", derive(clap::ValueEnum))]
pub enum ScriptHook {
    Pre,
    #[default]
    Post,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "value-enums", derive(clap::ValueEnum))]
pub enum ScriptEngine {
    #[default]
    Auto,
    #[cfg(feature = "rhai")]
    Rhai,
    #[cfg(feature = "starlark")]
    Starlark,
    #[cfg(feature = "lua")]
    Lua,
    Shell,
}

#[derive(Debug, Clone, Default)]
pub struct ScriptOptions<'a> {
    pub engine: ScriptEngine,
    pub rename: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct Script {
    engine: ScriptEngine,
}

impl Script {
    pub fn new(path: &Path) -> Result<Self, storage::Error> {
        Self::new_with_options(path, &ScriptOptions::default())
    }

    pub fn new_with_options(path: &Path, options: &ScriptOptions) -> Result<Self, storage::Error> {
        let name = path.display().to_string();
        // let dest = SCRIPTS_DIR.join(name);
        // storage::copy_data("script", path, &dest, false)?;
        Ok(Self {
            engine: options.engine,
        })
    }

    pub fn run(&self) -> Result<(), Error> {
        match self.engine {
            ScriptEngine::Auto => todo!(),
            #[cfg(feature = "rhai")]
            ScriptEngine::Rhai => todo!(),
            #[cfg(feature = "starlark")]
            ScriptEngine::Starlark => todo!(),
            #[cfg(feature = "lua")]
            ScriptEngine::Lua => todo!(),
            ScriptEngine::Shell => todo!(),
        }
        todo!()
    }
}
