use crate::storage;
use clap::{Args, Subcommand, ValueEnum};
use codablellm::Language;
use color_eyre::eyre::Result;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs,
    path::PathBuf,
    sync::{LazyLock, RwLock},
};
use strum::IntoEnumIterator;

pub static PATH: LazyLock<PathBuf> = LazyLock::new(|| storage::CONFIG_DIR.join("config.toml"));

static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| {
    RwLock::new(
        load()
            .inspect_err(|error| {
                tracing::error!(
                    %error,
                    "Failed to load CodableLLM config - using default configuration options"
                );
            })
            .unwrap_or_default(),
    )
});

fn load() -> Result<Config> {
    let config: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file(&*PATH))
        .merge(Env::prefixed("CODABLELLM_").split("_"))
        .extract()?;
    Ok(config)
}

pub fn get() -> Config {
    CONFIG.read().unwrap().clone()
}

pub fn update<F>(f: F) -> Result<()>
where
    F: FnOnce(&mut Config),
{
    let mut guard = CONFIG.write().unwrap();
    f(&mut guard);
    if let Some(parent) = PATH.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(&*guard)?;
    fs::write(&*PATH, contents)?;
    Ok(())
}

pub fn generate() -> Result<()> {
    update(|_| {})
}

pub fn reset() -> Result<()> {
    if PATH.exists() {
        fs::remove_file(&*PATH)?;
    }
    generate()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct Config {
    pub display: DisplayConfig,
    pub languages: LanguagesConfig,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            toml::to_string_pretty(self).unwrap_or_else(|_| format!("{:#?}", self))
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct DisplayConfig {
    pub progress: bool,
    pub console_log_level: LogLevel,
    pub file_log_level: LogLevel,
    pub interactive: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            progress: true,
            console_log_level: LogLevel::Off,
            file_log_level: LogLevel::Info,
            interactive: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum LogLevel {
    Off,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogLevel::Off => "off",
                LogLevel::Trace => "trace",
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warn => "warn",
                LogLevel::Error => "error",
            }
        )
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct LanguagesConfig {
    pub headers_as_cpp: bool,
    pub include: Vec<Language>,
}

impl Default for LanguagesConfig {
    fn default() -> Self {
        Self {
            headers_as_cpp: false,
            include: Language::iter().collect(),
        }
    }
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct ConfigArgs {
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
        #[arg(long)]
        unset: bool,
    },
    #[command(name = "display.console-log-level")]
    DisplayConsoleLogLevel {
        level: Option<LogLevel>,
        #[arg(long)]
        unset: bool,
    },
    #[command(name = "display.file-log-level")]
    DisplayFileLogLevel {
        level: Option<LogLevel>,
        #[arg(long)]
        unset: bool,
    },
    #[command(name = "forge.github-token")]
    ForgeGitHubToken {
        token: Option<String>,
        #[arg(long)]
        unset: bool,
    },
    #[command(name = "forge.gitlab-token")]
    ForgeGitLabToken {
        token: Option<String>,
        #[arg(long)]
        unset: bool,
    },
    #[command(name = "languages.headers-as-cpp")]
    LanguagesHeadersAsCpp {
        #[arg(action = clap::ArgAction::Set)]
        enable: Option<bool>,
        #[arg(long)]
        unset: bool,
    },
    #[command(name = "languages.include")]
    Languages {
        langs: Option<Vec<Language>>,
        #[arg(long)]
        unset: bool,
    },
}

pub fn run(command: ConfigArgs) -> Result<()> {
    if command.show_all {
        println!("{}", get());
        return Ok(());
    } else if command.show_path {
        println!("{}", PATH.display());
        return Ok(());
    }
    match command.command {
        _ => todo!(),
    }
    Ok(())
}
