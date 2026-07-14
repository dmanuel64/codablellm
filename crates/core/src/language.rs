use std::path::Path;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Display, EnumIter)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[cfg_attr(
    feature = "value-enums",
    derive(clap::ValueEnum, Serialize, Deserialize),
    serde(rename_all = "lowercase"),
    clap(rename_all = "lowercase")
)]
pub enum Language {
    C,
    #[strum(serialize = "c++")]
    #[cfg_attr(feature = "value-enums", clap(name = "c++"), serde(rename = "c++"))]
    Cpp,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Rust,
    Java,
    #[strum(serialize = "c#")]
    #[cfg_attr(feature = "value-enums", clap(name = "c#"), serde(rename = "c#"))]
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
