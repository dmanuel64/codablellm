use crate::storage::APP_DIRS;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::PathBuf,
    sync::{LazyLock, RwLock},
};
use thiserror::Error;

pub static PATH: LazyLock<PathBuf> =
    LazyLock::new(|| APP_DIRS.config_local_dir().join("config.toml"));

static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| {
    RwLock::new(
        Config::load()
            .inspect_err(|e| {
                log::debug!("{e}");
                log::error!(
                    "Failed to load CodableLLM config - using default configuration options"
                );
            })
            .unwrap_or_default(),
    )
});

pub fn get() -> Config {
    CONFIG.read().unwrap().clone()
}

pub fn update<F>(f: F) -> Result<(), Error>
where
    F: FnOnce(&mut Config),
{
    let mut guard = CONFIG.write().unwrap();
    f(&mut guard);
    guard.save()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        if !PATH.exists() {
            return Ok(Self::default());
        }
        toml::from_str(&fs::read_to_string(*PATH)?).into()
    }

    pub fn save(&self) -> Result<(), Error> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(&*PATH, contents)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("failed to serialize config data")]
    Serialization(#[from] toml::ser::Error),
    #[error("failed to deserialize config data")]
    Deserialization(#[from] toml::de::Error),
}
