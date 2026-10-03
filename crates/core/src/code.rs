use std::{
    cell::RefCell,
    convert::Infallible,
    fmt::Display,
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize};

use crate::{code::callable::Callable, language::Language};

pub mod callable;

thread_local! {
    static PARSER: Rc<RefCell<tree_sitter::Parser>> = Rc::new(RefCell::new(tree_sitter::Parser::new()));
}

pub trait Descriptor {
    fn text(&self) -> &str;
    fn language(&self) -> Language;
    fn file(&self) -> Option<&Path>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnparsedCode {
    pub text: String,
    pub language: Language,
    pub file: Option<PathBuf>,
}

impl UnparsedCode {
    pub fn new(text: String, language: Language) -> Self {
        Self {
            text,
            language,
            file: None,
        }
    }
}

impl Deref for UnparsedCode {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedCode {
    raw: UnparsedCode,
    #[serde(skip)]
    tree: Result<tree_sitter::Tree, Error>,
}

impl ParsedCode {
    pub fn new(&self, text: String, language: Language) -> Self {
        let raw = UnparsedCode::new(text, language);
        Self {
            raw,
            tree: PARSER.with(|p| {
                let parser = p.borrow_mut();
                parser.set_language(&raw.language)?;
                parser.parse(text, None).map_err(Error::Parsing)
            }),
        }
    }
}

impl Deref for ParsedCode {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Code {
    Unparsed(UnparsedCode),
    Parsed(ParsedCode),
}

impl Code {
    pub fn new(text: String) -> Self {
        Self::Unparsed { text, file: None }
    }

    pub fn read_file(file: PathBuf) -> Result<Self, Error> {
        let text = fs::read_to_string(file)?;
        Ok(Self::Unparsed {
            text,
            file: Some(file),
        })
    }

    pub fn parse_file(file: PathBuf, language: &tree_sitter::Language) -> Result<Self, Error> {
        Self::read_file(file)?.into_parsed(language)
    }

    pub fn text(&self) -> &str {
        match self {
            Code::Unparsed { text, .. } | Code::Parsed { text, .. } => text,
        }
    }

    pub fn file(&self) -> Option<&Path> {
        match self {
            Code::Unparsed { file, .. } | Code::Parsed { file, .. } => {
                file.as_ref().map(PathBuf::as_path)
            }
        }
    }

    pub fn callables(&self) -> Vec<Callable> {
        todo!()
    }

    pub fn into_parsed(self, language: &tree_sitter::Language) -> Result<Self, Error> {
        let text = match self {
            Code::Unparsed { text, .. } | Code::Parsed { text, .. } => text,
        };
        let file = match self {
            Code::Unparsed { file, .. } | Code::Parsed { file, .. } => file,
        };
        let tree = PARSER.try_with(|p| {
            let parser = p.borrow_mut();
            parser.set_language(language)?;
            parser
                .parse(self.text(), None)
                .ok_or_else(|| Error::Parsing)
        })?;
        Ok(Code::Parsed { text, file, tree })
    }
}

impl Deref for Code {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.text()
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = self.text();
        write!(f, "{text}")
    }
}

impl FromStr for Code {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Code::Unparsed {
            text: s.to_string(),
            file: None,
        })
    }
}
