use std::path::Path;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Language {
    Source(SourceLanguage),
    Assembly(),
    PseudoC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[cfg_attr(
    feature = "value-enums",
    derive(clap::ValueEnum),
    clap(rename_all = "lowercase")
)]
pub enum SourceLanguage {
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

impl SourceLanguage {
    pub fn file_extensions(&self) -> &'static [&'static str] {
        match self {
            SourceLanguage::C => &["c", "h"],
            SourceLanguage::Cpp => &["cpp", "cxx", "cc", "c++", "hpp", "hxx", "hh", "h++", "h"],
            SourceLanguage::Python => &["py", "pyw"],
            SourceLanguage::JavaScript => &["js", "mjs", "cjs", "jsx"],
            SourceLanguage::TypeScript => &["ts", "tsx"],
            SourceLanguage::Go => &["go"],
            SourceLanguage::Rust => &["rs"],
            SourceLanguage::Java => &["java"],
            SourceLanguage::CSharp => &["cs"],
        }
    }

    pub fn is_compilable(&self) -> bool {
        match self {
            SourceLanguage::C
            | SourceLanguage::Cpp
            | SourceLanguage::Rust
            | SourceLanguage::Go
            | SourceLanguage::Java
            | SourceLanguage::CSharp => true,
            SourceLanguage::Python | SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                false
            }
        }
    }

    pub fn from_path(path: &Path) -> Option<SourceLanguage> {
        let ext = path.extension()?.to_str()?;
        Self::iter().find(|l| l.file_extensions().contains(&ext))
    }
}

impl From<SourceLanguage> for tree_sitter::Language {
    fn from(value: SourceLanguage) -> Self {
        match value {
            SourceLanguage::C => tree_sitter_c::LANGUAGE.into(),
            SourceLanguage::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            SourceLanguage::Python => tree_sitter_python::LANGUAGE.into(),
            SourceLanguage::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            SourceLanguage::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            SourceLanguage::Go => tree_sitter_go::LANGUAGE.into(),
            SourceLanguage::Rust => tree_sitter_rust::LANGUAGE.into(),
            SourceLanguage::Java => tree_sitter_java::LANGUAGE.into(),
            SourceLanguage::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
        }
    }
}
