use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use polars::prelude::*;
use thiserror::Error;

use crate::{config, function::Function};

pub static DATASETS_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| config::APP_DIRS.data_local_dir().join("datasets"));

pub fn ensure_datasets_dir_exists() -> Result<(), config::Error> {
    config::ensure_dir_exists(&DATASETS_DIR)
}

pub static SCRIPTS_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| config::APP_DIRS.data_local_dir().join("scripts"));

pub fn ensure_scripts_dir_exists() -> Result<(), config::Error> {
    config::ensure_dir_exists(&SCRIPTS_DIR)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create dataset: {0}")]
    DatasetCreation(#[source] PolarsError),
    #[error("script failure")]
    ScriptError,
}

pub trait Dataset {
    fn df(&self) -> &DataFrame;
    fn df_mut(&mut self) -> &mut DataFrame;
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
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum ScriptHook {
    Pre,
    #[default]
    Post,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
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
    pub hook: ScriptHook,
    pub engine: ScriptEngine,
    pub rename: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct Script {
    pub hook: ScriptHook,
    engine: ScriptEngine,
}

impl Script {
    pub fn new(path: &Path) -> Result<Self, config::Error> {
        Self::new_with_options(path, &ScriptOptions::default())
    }

    pub fn new_with_options(path: &Path, options: &ScriptOptions) -> Result<Self, config::Error> {
        ensure_scripts_dir_exists()?;
        let name = path.to_string_lossy().to_string();
        let dest = SCRIPTS_DIR.join(name);
        config::copy_data("script", path, &dest, false)?;
        Ok(Self {
            hook: options.hook,
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
