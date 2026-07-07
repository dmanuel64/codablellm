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

fn build_figment() -> Figment {
    Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file(&*PATH).nested())
        .merge(Env::prefixed("CODABLELLM_").global())
}

struct State {
    figment: Figment,
    config: Config,
}

use figment::{
    Figment, Profile,
    providers::{Env, Format, Serialized, Toml},
};
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

pub static PATH: LazyLock<PathBuf> = LazyLock::new(|| storage::CONFIG_DIR.join("config.toml"));

/// Builds defaults -> file -> env, with `.nested()` so each top-level
/// TOML table (`[dev]`, `[prod]`, etc.) is treated as its own profile.
fn build_figment() -> Figment {
    Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file(&*PATH).nested())
        .merge(Env::prefixed("CODABLELLM_").global())
}

struct State {
    figment: Figment,
    config: Config,
}

impl State {
    fn load() -> Self {
        let figment = build_figment();

        // Which profile to select: reserved top-level "profile" key in the
        // file (falls back to Figment's Default profile).
        let profile = figment
            .extract_inner::<String>("profile")
            .map(Profile::new)
            .unwrap_or_else(|_| Profile::Default);

        let figment = figment.select(profile);

        let config = figment
            .extract()
            .inspect_err(|error| {
                tracing::error!(
                    %error,
                    "Failed to load CodableLLM config - using default configuration options"
                );
            })
            .unwrap_or_default();

        Self { figment, config }
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = PATH.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Read the existing file so other profiles aren't clobbered.
        let mut doc: toml::Table = std::fs::read_to_string(PATH)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default();

        doc.insert(
            self.figment.profile().to_string(),
            toml::Value::try_from(&self.config)?,
        );
        doc.insert(
            "profile".into(),
            toml::Value::String(self.figment.profile().to_string()),
        );

        std::fs::write(&*PATH, toml::to_string_pretty(&doc)?)?;
        Ok(())
    }
}

static STATE: LazyLock<RwLock<State>> = LazyLock::new(|| RwLock::new(State::load()));

pub fn get() -> Config {
    STATE.read().unwrap().config.clone()
}

pub fn current_profile() -> Profile {
    STATE.read().unwrap().figment.profile().clone()
}

pub fn set_profile(profile: impl Into<Profile>) -> Result<()> {
    let mut state = STATE.write().unwrap();
    state.figment = state.figment.clone().select(profile.into());
    state.config = state.figment.extract()?;
    state.save()
}

pub fn update<F>(f: F) -> Result<()>
where
    F: FnOnce(&mut Config),
{
    let mut state = STATE.write().unwrap();
    f(&mut state.config);
    state.save()
}

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct Config {
    pub display: DisplayConfig,
    pub forge: ForgeConfig,
    pub languages: LanguagesConfig,
}

impl Config {
    fn load() -> Result<Self> {
        if !PATH.exists() {
            return Ok(Self::default());
        }
        let config = toml::from_str(&fs::read_to_string(&*PATH)?)?;
        Ok(config)
    }

    fn save(&self) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(&*PATH, contents)?;
        Ok(())
    }
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
#[serde(rename_all = "kebab-case", default)]
pub struct ForgeConfig {
    pub github_token: Option<String>,
    pub gitlab_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct LanguagesConfig {
    pub headers_as_cpp: bool,
    pub include: Vec<String>,
}

impl Default for LanguagesConfig {
    fn default() -> Self {
        Self {
            headers_as_cpp: false,
            include: Language::iter().map(|l| l.to_string()).collect(),
        }
    }
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

pub fn run(command: Command) -> Result<()> {
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
