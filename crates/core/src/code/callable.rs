pub mod associated_function;
pub mod function;
pub mod method;

use std::{cell::RefCell, fmt::Display, ops::Range, path::PathBuf, rc::Rc, str::Utf8Error};

use indoc::indoc;
use serde::{Deserialize, Serialize};
use tree_sitter::StreamingIterator;

use associated_function::SourceAssociatedFunction;
use function::{Function, SourceFunction};
use method::SourceMethod;

use crate::{
    SourceLanguage,
    code::Code,
    language::Language,
    parser::{Error, ParsedCode},
};

const fn get_function_sexp(language: &SourceLanguage) -> &'static str {
    match language {
        SourceLanguage::C => indoc! {r#"
            (function_definition
                declarator: (function_declarator
                    declarator: (identifier) @name)
            ) @definition
        "#},
        SourceLanguage::Cpp => indoc! {r#"
            (function_definition
                declarator: (function_declarator
                    declarator: [(identifier) (field_identifier)] @name)
            ) @definition
        "#},
        SourceLanguage::Python => indoc! {r#"
            (function_definition
                name: (identifier) @name
            ) @definition
        "#},
        SourceLanguage::JavaScript => indoc! {r#"
            [
                (function_declaration
                    name: (identifier) @name) @definition
                (method_definition
                    name: (property_identifier) @name) @definition
            ]
        "#},
        SourceLanguage::TypeScript => indoc! {r#"
            [
                (function_declaration
                    name: (identifier) @name) @definition
                (method_definition
                    name: (property_identifier) @name) @definition
            ]
        "#},
        SourceLanguage::Go => indoc! {r#"
            (function_declaration
                name: (identifier) @name
            ) @definition
            (method_declaration
                name: (field_identifier) @name
            ) @definition
        "#},
        SourceLanguage::Rust => indoc! {r#"
            (function_item
                name: (identifier) @name
            ) @definition
        "#},
        SourceLanguage::Java => indoc! {r#"
            (method_declaration
                name: (identifier) @name
            ) @definition
        "#},
        SourceLanguage::CSharp => indoc! {r#"
            (method_declaration
                name: (identifier) @name
            ) @definition
        "#},
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub source: PathBuf,
    #[serde(skip)]
    pub bytes_range: Range<usize>,
    pub line_range: Range<usize>,
    pub column_range: Range<usize>,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source = self.source.to_string_lossy();
        write!(f, "{source}:{}", self.line_range.start)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Descriptor {
    pub name: String,
    pub definition: Code,
    pub language: Language,
    pub location: Option<Location>,
}

thread_local! {
    static PARSER: RefCell<tree_sitter::Parser> = RefCell::new({
        tree_sitter::Parser::new()
    });
}

trait ParsedFunction: Function {
    fn tree(&self) -> Rc<RefCell<tree_sitter::Tree>>;

    fn node(&self) -> Result<tree_sitter::Node, Error> {
        let end = self
            .descriptor()
            .location
            .as_ref()
            .map(|l| l.bytes_range.end)
            .unwrap_or(0);
        self.tree()
            .borrow()
            .root_node()
            .descendant_for_byte_range(
                self.descriptor()
                    .location
                    .as_ref()
                    .map(|l| l.bytes_range.start)
                    .unwrap_or(0),
                end,
            )
            .ok_or_else(|| todo!())
    }

    fn query_with<F, R>(&self, sexp: &str, f: F) -> Result<R, Error>
    where
        F: FnOnce(&tree_sitter::Query, tree_sitter::QueryMatches<'_, '_, &[u8], &[u8]>) -> R,
    {
        let tree = self.tree();
        let tree = tree.borrow();
        let root_node = tree.root_node();
        let query = tree_sitter::Query::new(&root_node.language(), sexp).map_err(Error::Query)?;
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches = cursor.matches(&query, root_node, self.definition().as_bytes());
        Ok(f(&query, matches))
    }
}

/// Any callable extracted from source: a free function, a method, or an
/// associated (static) function - each spanning all supported languages via
/// its own `Source*` enum. `Method`/`AssociatedFunction` don't apply to
/// `Decompiled`/`Assembly` output (no receiver/scope semantics in machine
/// code), which is why this stays its own type instead of folding into
/// `Callable` directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceCallable {
    Function(SourceFunction),
    Method(SourceMethod),
    AssociatedFunction(SourceAssociatedFunction),
}

impl Function for SourceCallable {
    fn descriptor(&self) -> &Descriptor {
        match self {
            SourceCallable::Function(func) => func.descriptor(),
            SourceCallable::Method(func) => func.descriptor(),
            SourceCallable::AssociatedFunction(func) => func.descriptor(),
        }
    }
}

impl SourceCallable {
    pub fn as_function(&self) -> Option<&SourceFunction> {
        if let SourceCallable::Function(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_method(&self) -> Option<&SourceMethod> {
        if let SourceCallable::Method(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_associated_function(&self) -> Option<&SourceAssociatedFunction> {
        if let SourceCallable::AssociatedFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }
}

impl Display for SourceCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceCallable::Function(func) => write!(f, "{func}"),
            SourceCallable::Method(func) => write!(f, "{func}"),
            SourceCallable::AssociatedFunction(func) => write!(f, "{func}"),
        }
    }
}

/// Any callable, in any representation. Currently only ever holds `Source`;
/// `Assembly`/`Decompiled` variants land once those types carry real data
/// (a `Descriptor` and a `Function` impl) instead of being empty stubs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Callable {
    Source(SourceCallable),
}

impl Function for Callable {
    fn descriptor(&self) -> &Descriptor {
        match self {
            Callable::Source(func) => func.descriptor(),
        }
    }
}

impl Callable {
    pub fn as_source(&self) -> Option<&SourceCallable> {
        if let Callable::Source(func) = self {
            Some(func)
        } else {
            None
        }
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::Source(func) => write!(f, "{func}"),
        }
    }
}
