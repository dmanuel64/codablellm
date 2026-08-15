use std::path::Path;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[cfg_attr(
    feature = "value-enums",
    derive(clap::ValueEnum),
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

impl From<Language> for tree_sitter::Language {
    fn from(value: Language) -> Self {
        match value {
            Language::C => tree_sitter_c::LANGUAGE.into(),
            Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::Go => tree_sitter_go::LANGUAGE.into(),
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::Java => tree_sitter_java::LANGUAGE.into(),
            Language::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
        }
    }
}
