use std::path::Path;

use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Default)]
pub struct Options {
    pub display_progress: bool,
    pub request_builder: Option<reqwest::blocking::ClientBuilder>,
}

#[derive(Debug, Clone, Display, EnumIter)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Language {
    C,
    #[strum(serialize = "C++")]
    #[cfg_attr(feature = "clap", clap(name = "c++"))]
    Cpp,
    Python,
    #[cfg_attr(feature = "clap", clap(name = "javascript"))]
    JavaScript,
    #[cfg_attr(feature = "clap", clap(name = "typescript"))]
    TypeScript,
    Go,
    Rust,
    Java,
    #[strum(serialize = "C#")]
    #[cfg_attr(feature = "clap", clap(name = "c#"))]
    CSharp,
}

impl Language {
    pub fn file_extensions(&self) -> &'static [&'static str] {
        match self {
            Language::C => &["c", "h"],
            Language::Cpp => &["cpp", "cxx", "cc", "c++", "hpp", "hxx", "hh", "h++", "h"],
            Language::Python => &["py", "pyw"],
            Language::JavaScript => &["js", "mjs", "cjs", "jsx"],
            Language::TypeScript => &["ts", "tsx"],
            Language::Go => &["go"],
            Language::Rust => &["rs"],
            Language::Java => &["java"],
            Language::CSharp => &["cs"],
        }
    }

    pub fn is_compiled(&self) -> bool {
        match self {
            Language::C
            | Language::Cpp
            | Language::Rust
            | Language::Go
            | Language::Java
            | Language::CSharp => true,
            Language::Python | Language::JavaScript | Language::TypeScript => false,
        }
    }

    pub fn from_path(path: &Path) -> Option<Language> {
        let ext = path.extension()?.to_str()?;
        Self::iter().find(|l| l.file_extensions().contains(&ext))
    }
}
