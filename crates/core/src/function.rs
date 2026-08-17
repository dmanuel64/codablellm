use std::{
    ffi::OsStr,
    fmt::Display,
    ops::Range,
    path::{Path, PathBuf},
    str::Utf8Error,
};

use serde::{Deserialize, Serialize};
use tree_sitter::StreamingIterator;

use crate::{
    Language,
    parser::{Error, ParsedCode},
};

const C_FUNCTION_SEXP: &str = r#"
        (function_definition
            declarator: (function_declarator
                declarator: (identifier) @name)
        ) @definition
    "#;
const CPP_FUNCTION_SEXP: &str = r#"
        (function_definition
            declarator: (function_declarator
                declarator: [(identifier) (field_identifier)] @name)
        ) @definition
    "#;
const PYTHON_FUNCTION_SEXP: &str = r#"
        (function_definition
            name: (identifier) @name
        ) @definition
    "#;
const JAVASCRIPT_FUNCTION_SEXP: &str = r#"
        [
            (function_declaration
                name: (identifier) @name) @definition
            (method_definition
                name: (property_identifier) @name) @definition
        ]
    "#;
const TYPESCRIPT_FUNCTION_SEXP: &str = r#"
        [
            (function_declaration
                name: (identifier) @name) @definition
            (method_definition
                name: (property_identifier) @name) @definition
        ]
    "#;
const GO_FUNCTION_SEXP: &str = r#"
        (function_declaration
            name: (identifier) @name
        ) @definition
        (method_declaration
            name: (field_identifier) @name
        ) @definition
    "#;
const RUST_FUNCTION_SEXP: &str = r#"
        (function_item
            name: (identifier) @name
        ) @definition
    "#;
const JAVA_FUNCTION_SEXP: &str = r#"
        (method_declaration
            name: (identifier) @name
        ) @definition
    "#;
const CSHARP_FUNCTION_SEXP: &str = r#"
        (method_declaration
            name: (identifier) @name
        ) @definition
    "#;

// TODO: metadata isn't the most accurate name to include name & definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    name: String,
    definition: String,
    extra: Option<serde_json::Value>,
    source: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    #[serde(skip)]
    bytes_range: Range<usize>,
    line_range: Range<usize>,
    column_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Function {
    // TODO: make these variants types so they have non-public access qualifiers
    Source {
        metadata: Metadata,
        language: Language,
        location: Location,
    },
    Decompiled {
        metadata: Metadata,
    },
    Assembly {
        metadata: Metadata,
        location: Location,
    },
}

impl Function {
    pub fn new_source(
        name: String,
        definition: String,
        source: Option<PathBuf>,
        language: Language,
        bytes_range: Range<usize>,
        line_range: Range<usize>,
        column_range: Range<usize>,
    ) -> Self {
        Self::Source {
            metadata: Metadata {
                name,
                definition,
                extra: None,
                source,
            },
            language,
            location: Location {
                bytes_range,
                line_range,
                column_range,
            },
        }
    }

    pub fn new_assembly(
        name: String,
        definition: String,
        source: Option<PathBuf>,
        bytes_range: Range<usize>,
        line_range: Range<usize>,
        column_range: Range<usize>,
    ) -> Self {
        Self::Assembly {
            metadata: Metadata {
                name,
                definition,
                extra: None,
                source,
            },
            location: Location {
                bytes_range,
                line_range,
                column_range,
            },
        }
    }

    pub fn new_decompiled(name: String, definition: String, source: Option<PathBuf>) -> Self {
        Self::Decompiled {
            metadata: Metadata {
                name,
                definition,
                extra: None,
                source,
            },
        }
    }

    pub fn metadata(&self) -> &Metadata {
        match self {
            Function::Source { metadata, .. } => metadata,
            Function::Decompiled { metadata, .. } => metadata,
            Function::Assembly { metadata, .. } => metadata,
        }
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        match self {
            Function::Source { metadata, .. } => metadata,
            Function::Decompiled { metadata, .. } => metadata,
            Function::Assembly { metadata, .. } => metadata,
        }
    }

    pub fn name(&self) -> &str {
        &self.metadata().name
    }

    pub fn definition(&self) -> &str {
        &self.metadata().definition
    }

    pub fn source(&self) -> Option<&Path> {
        self.metadata().source.as_ref().map(PathBuf::as_path)
    }

    pub fn language(&self) -> String {
        match self {
            Function::Source { language, .. } => language.to_string(),
            Function::Decompiled { .. } => String::from("Pseudo-C"),
            Function::Assembly { .. } => String::from("Assembly"),
        }
    }

    pub fn extra(&self) -> &Option<serde_json::Value> {
        &self.metadata().extra
    }

    pub fn extra_mut(&mut self) -> &mut Option<serde_json::Value> {
        &mut self.metadata_mut().extra
    }

    pub fn edit(&mut self, new_name: Option<String>, new_definition: String) {
        match self {
            Function::Source {
                metadata, location, ..
            }
            | Function::Assembly { metadata, location } => {
                let Metadata {
                    name, definition, ..
                } = metadata;
                let Location {
                    bytes_range,
                    line_range,
                    column_range,
                } = location;
                line_range.end = line_range.start + new_definition.lines().count();
                column_range.end = new_definition
                    .lines()
                    .map(str::len)
                    .max()
                    .unwrap_or_default();
                bytes_range.end = bytes_range.start + new_definition.bytes().len();
                if let Some(n) = new_name {
                    *name = n;
                }
                *definition = new_definition;
            }
            Function::Decompiled { metadata } => todo!("support for decompiled code edits"),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Source { location, .. } | Function::Assembly { location, .. } => write!(
                f,
                "{}::{}:{}",
                self.source()
                    .and_then(Path::file_name)
                    .map(OsStr::to_string_lossy)
                    .unwrap_or_else(|| String::from("<MEM>").into()),
                self.name(),
                location.line_range.start
            ),
            Function::Decompiled { .. } => write!(
                f,
                "{}::{}",
                self.source()
                    .and_then(Path::file_name)
                    .map(OsStr::to_string_lossy)
                    .unwrap_or_else(|| String::from("<MEM>").into()),
                self.name(),
            ),
        }
    }
}

#[cfg(feature = "rhai")]
impl rhai::CustomType for Function {
    fn build(mut builder: rhai::TypeBuilder<Self>) {
        builder
            .with_name("Function")
            .with_get_set(
                "name",
                |func: &mut Self| func.name().to_string(),
                |func: &mut Self, val: String| {
                    func.metadata_mut().name = val;
                },
            )
            .with_get("language", |func: &mut Self| func.language())
            .with_get_set(
                "definition",
                |func: &mut Self| func.definition().to_string(),
                |func: &mut Self, val: String| {
                    func.edit(None, val);
                },
            );
    }
}

pub(crate) struct ParsedFunctions {
    code: ParsedCode,
    functions: Vec<Function>,
}

impl ParsedFunctions {
    pub fn new(code: ParsedCode) -> Self {
        Self {
            code,
            functions: Vec::new(),
        }
    }

    pub fn code(&self) -> &ParsedCode {
        &self.code
    }

    fn functions_inner(&mut self) -> Result<Vec<Function>, Error> {
        let sexp = match self.code.language() {
            Language::C => C_FUNCTION_SEXP,
            Language::Cpp => CPP_FUNCTION_SEXP,
            Language::Python => PYTHON_FUNCTION_SEXP,
            Language::JavaScript => JAVASCRIPT_FUNCTION_SEXP,
            Language::TypeScript => TYPESCRIPT_FUNCTION_SEXP,
            Language::Go => GO_FUNCTION_SEXP,
            Language::Rust => RUST_FUNCTION_SEXP,
            Language::Java => JAVA_FUNCTION_SEXP,
            Language::CSharp => CSHARP_FUNCTION_SEXP,
        };

        let language = *self.code.language();
        let source = self.code.source.clone();
        let code = self.code.code().to_vec();

        let (query, mut matches) = self.code.query(sexp)?;
        let name_idx = query
            .capture_index_for_name("name")
            .expect("The s-expression to contain the name capture group");
        let definition_idx = query
            .capture_index_for_name("definition")
            .expect("The s-expression to contain the definition capture group");
        let mut functions = Vec::new();
        while let Some(m) = matches.next() {
            let name_capture = m.captures.iter().find(|c| c.index == name_idx);
            let definition_capture = m.captures.iter().find(|c| c.index == definition_idx);
            if let (Some(name), Some(def)) = (name_capture, definition_capture) {
                let name = name
                    .node
                    .utf8_text(&code)
                    .map_err(Utf8Error::from)?
                    .to_string();
                let definition = def
                    .node
                    .utf8_text(&code)
                    .map_err(Utf8Error::from)?
                    .to_string();
                let range = def.node.range();
                let bytes_range = def.node.byte_range();
                let line_range = range.start_point.row..range.end_point.row;
                let column_range = range.start_point.column..range.end_point.column;
                functions.push(Function::new_source(
                    name,
                    definition,
                    source.clone(),
                    language,
                    bytes_range,
                    line_range,
                    column_range,
                ));
            }
        }
        Ok(functions)
    }

    pub fn functions(&mut self) -> Result<&[Function], Error> {
        if self.functions.is_empty() {
            self.functions = self.functions_inner()?;
        }
        Ok(&self.functions)
    }

    pub fn edit<EditFn>(&mut self, e: EditFn) -> Result<(), Error>
    where
        EditFn: Fn(&mut Function),
    {
        if self.functions.is_empty() {
            self.functions = self.functions_inner()?;
        }
        // Split into disjoint borrows so `code.edit()` can run per-function
        // inside the loop below without conflicting with the loop's own
        // borrow of `functions`.
        let Self { code, functions } = self;

        // Process back-to-front by byte position so editing one function
        // never shifts the still-stale offsets of another we haven't
        // gotten to yet.
        let mut order: Vec<usize> = (0..functions.len()).collect();
        order.sort_unstable_by_key(|&i| {
            std::cmp::Reverse(match &functions[i] {
                Function::Source { location, .. } | Function::Assembly { location, .. } => {
                    location.bytes_range.start
                }
                Function::Decompiled { .. } => 0,
            })
        });

        for i in order {
            let function = &mut functions[i];
            let old_definition = function.definition().to_string();
            e(function);
            if function.definition() != old_definition {
                let start = match function {
                    Function::Source { location, .. } | Function::Assembly { location, .. } => {
                        Some(location.bytes_range.start)
                    }
                    Function::Decompiled { .. } => None,
                };
                if let Some(start) = start {
                    let end = start + old_definition.len();
                    let new_definition = function.definition().to_string();
                    code.edit(|bytes| {
                        bytes.splice(start..end, new_definition.into_bytes());
                    })?;
                }
            }
        }
        Ok(())
    }
}

impl From<ParsedCode> for ParsedFunctions {
    fn from(value: ParsedCode) -> Self {
        Self::new(value)
    }
}

impl From<ParsedFunctions> for ParsedCode {
    fn from(value: ParsedFunctions) -> Self {
        value.code
    }
}
