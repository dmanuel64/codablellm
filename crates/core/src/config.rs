use crate::storage;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::PathBuf,
    sync::{LazyLock, RwLock},
};
use thiserror::Error;

pub static PATH: LazyLock<PathBuf> = LazyLock::new(|| storage::CONFIG_DIR.join("config.toml"));

static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| {
    RwLock::new(
        Config::load()
            .inspect_err(|error| {
                tracing::error!(
                    %error,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub display: DisplayConfig,
    pub forge: ForgeConfig,
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        if !PATH.exists() {
            return Ok(Self::default());
        }
        toml::from_str(&fs::read_to_string(&*PATH)?).map_err(Error::from)
    }

    pub fn save(&self) -> Result<(), Error> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(&*PATH, contents)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub progress: bool,
    pub console_log_level: LogLevel,
    pub file_log_level: LogLevel,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            progress: true,
            console_log_level: LogLevel::Off,
            file_log_level: LogLevel::Info,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "value-enums", derive(clap::ValueEnum))]
pub enum LogLevel {
    Off,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        match self {
            LogLevel::Off => "off",
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
        .to_string()
    }
}

impl From<LogLevel> for u8 {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Off => 0,
            LogLevel::Trace => 1,
            LogLevel::Debug => 2,
            LogLevel::Info => 3,
            LogLevel::Warn => 4,
            LogLevel::Error => 5,
        }
    }
}

impl From<u8> for LogLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => LogLevel::Off,
            1 => LogLevel::Trace,
            2 => LogLevel::Debug,
            3 => LogLevel::Info,
            4 => LogLevel::Warn,
            _ => LogLevel::Error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForgeConfig {
    pub github_token: Option<String>,
    pub gitlab_token: Option<String>,
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
