use std::{
    any::Any,
    ffi::OsStr,
    fmt::Display,
    ops::Range,
    path::{Path, PathBuf},
    str::Utf8Error,
};

use indoc::indoc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tree_sitter::StreamingIterator;

use crate::{
    language::{self, Language},
    parser::ParsedCode,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Incorrect function language")]
    InvalidLanguage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFunction<L: Language> {
    name: String,
    definition: String,
    source: Option<PathBuf>,
    language: L,
    #[serde(skip)]
    bytes_range: Range<usize>,
    line_range: Range<usize>,
    column_range: Range<usize>,
}

impl<L: Language + 'static> From<SourceFunction<L>> for AnySourceFunction {
    fn from(
        SourceFunction {
            name,
            definition,
            source,
            language,
            bytes_range,
            line_range,
            column_range,
        }: SourceFunction<L>,
    ) -> Self {
        Self {
            name,
            definition,
            source,
            language: Box::new(language),
            bytes_range,
            line_range,
            column_range,
        }
    }
}

pub struct AnySourceFunction {
    name: String,
    definition: String,
    source: Option<PathBuf>,
    language: Box<dyn Language>,
    bytes_range: Range<usize>,
    line_range: Range<usize>,
    column_range: Range<usize>,
}

impl AnySourceFunction {
    fn into_language<L: Language + Clone>(self) -> Result<SourceFunction<L>, Error> {
        let any_obj: Box<dyn Any> = self.language;
        if let Ok(lang) = any_obj.downcast::<L>() {
            Ok(SourceFunction {
                name: self.name,
                definition: self.definition,
                source: self.source,
                language: *lang,
                bytes_range: self.bytes_range,
                line_range: self.line_range,
                column_range: self.column_range,
            })
        } else {
            Err(Error::InvalidLanguage)
        }
    }

    fn as_language<L: Language + Clone>(&self) -> Option<SourceFunction<L>> {
        let any_obj: &dyn Any = &self.language;
        if let Some(lang) = any_obj.downcast_ref::<L>() {
            Some(SourceFunction {
                name: self.name.clone(),
                definition: self.definition.clone(),
                source: self.source.clone(),
                language: lang.clone(),
                bytes_range: self.bytes_range.clone(),
                line_range: self.line_range.clone(),
                column_range: self.column_range.clone(),
            })
        } else {
            None
        }
    }

    pub fn as_c(&self) -> Option<SourceFunction<language::C>> {
        self.as_language()
    }

    pub fn as_cpp(&self) -> Option<SourceFunction<language::Cpp>> {
        self.as_language()
    }

    pub fn as_python(&self) -> Option<SourceFunction<language::Python>> {
        self.as_language()
    }

    pub fn as_javascript(&self) -> Option<SourceFunction<language::JavaScript>> {
        self.as_language()
    }

    pub fn as_typescript(&self) -> Option<SourceFunction<language::TypeScript>> {
        self.as_language()
    }

    pub fn as_java(&self) -> Option<SourceFunction<language::Java>> {
        self.as_language()
    }

    pub fn as_go(&self) -> Option<SourceFunction<language::Go>> {
        self.as_language()
    }

    pub fn as_rust(&self) -> Option<SourceFunction<language::Rust>> {
        self.as_language()
    }

    pub fn as_csharp(&self) -> Option<SourceFunction<language::CSharp>> {
        self.as_language()
    }
}

impl TryFrom<AnySourceFunction> for SourceFunction<language::C> {
    type Error = Error;

    fn try_from(value: AnySourceFunction) -> Result<Self, Self::Error> {
        value.into_language()
    }
}

impl TryFrom<AnySourceFunction> for SourceFunction<language::Cpp> {
    type Error = Error;

    fn try_from(value: AnySourceFunction) -> Result<Self, Self::Error> {
        value.into_language()
    }
}

impl TryFrom<AnySourceFunction> for SourceFunction<language::Python> {
    type Error = Error;

    fn try_from(value: AnySourceFunction) -> Result<Self, Self::Error> {
        value.into_language()
    }
}

impl TryFrom<AnySourceFunction> for SourceFunction<language::JavaScript> {
    type Error = Error;

    fn try_from(value: AnySourceFunction) -> Result<Self, Self::Error> {
        value.into_language()
    }
}

impl TryFrom<AnySourceFunction> for SourceFunction<language::TypeScript> {
    type Error = Error;

    fn try_from(value: AnySourceFunction) -> Result<Self, Self::Error> {
        value.into_language()
    }
}

impl TryFrom<AnySourceFunction> for SourceFunction<language::Go> {
    type Error = Error;

    fn try_from(value: AnySourceFunction) -> Result<Self, Self::Error> {
        value.into_language()
    }
}

impl TryFrom<AnySourceFunction> for SourceFunction<language::Rust> {
    type Error = Error;

    fn try_from(value: AnySourceFunction) -> Result<Self, Self::Error> {
        value.into_language()
    }
}

impl TryFrom<AnySourceFunction> for SourceFunction<language::Java> {
    type Error = Error;

    fn try_from(value: AnySourceFunction) -> Result<Self, Self::Error> {
        value.into_language()
    }
}

impl TryFrom<AnySourceFunction> for SourceFunction<language::CSharp> {
    type Error = Error;

    fn try_from(value: AnySourceFunction) -> Result<Self, Self::Error> {
        value.into_language()
    }
}

trait FunctionSexpExt {
    fn function_sexp(&self) -> &str;
}

impl FunctionSexpExt for language::C {
    fn function_sexp(&self) -> &str {
        indoc! {r#"
            (function_definition
                 declarator: (function_declarator
                     declarator: (identifier) @name)
             ) @definition
        "#}
    }
}

impl FunctionSexpExt for language::Cpp {
    fn function_sexp(&self) -> &str {
        indoc! {r#"
            (function_definition
                declarator: (function_declarator
                    declarator: [(identifier) (field_identifier)] @name)
            ) @definition
        "#}
    }
}

impl FunctionSexpExt for language::Python {
    fn function_sexp(&self) -> &str {
        indoc! {r#"
            (function_definition
                name: (identifier) @name
            ) @definition
        "#}
    }
}

impl FunctionSexpExt for language::JavaScript {
    fn function_sexp(&self) -> &str {
        indoc! {r#"
            [
                (function_declaration
                    name: (identifier) @name) @definition
                (method_definition
                    name: (property_identifier) @name) @definition
            ]
        "#}
    }
}

impl FunctionSexpExt for language::TypeScript {
    fn function_sexp(&self) -> &str {
        indoc! {r#"
            [
                (function_declaration
                    name: (identifier) @name) @definition
                (method_definition
                    name: (property_identifier) @name) @definition
            ]
        "#}
    }
}

impl FunctionSexpExt for language::Go {
    fn function_sexp(&self) -> &str {
        indoc! {r#"
            (function_declaration
                name: (identifier) @name
            ) @definition
            (method_declaration
                name: (field_identifier) @name
            ) @definition
        "#}
    }
}

impl FunctionSexpExt for language::Rust {
    fn function_sexp(&self) -> &str {
        indoc! {r#"
            (function_item
                name: (identifier) @name
            ) @definition
        "#}
    }
}

impl FunctionSexpExt for language::Java {
    fn function_sexp(&self) -> &str {
        indoc! {r#"
            (method_declaration
                name: (identifier) @name
            ) @definition
        "#}
    }
}

impl FunctionSexpExt for language::CSharp {
    fn function_sexp(&self) -> &str {
        indoc! {r#"
            (method_declaration
                name: (identifier) @name
            ) @definition
        "#}
    }
}

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
        language: Metadata,
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
        language: Metadata,
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

pub(crate) struct ParsedSourceFunctions<L: Language> {
    code: ParsedCode<L>,
    functions: Vec<Function>,
}

impl<L: Language> ParsedSourceFunctions<L> {
    pub fn new(code: ParsedCode<L>) -> Self {
        Self {
            code,
            functions: Vec::new(),
        }
    }

    pub fn code(&self) -> &ParsedCode<L> {
        &self.code
    }

    fn functions_inner(&mut self) -> Result<Vec<Function>, Error> {
        let language = *self.code.language();
        let sexp = get_function_sexp(&language);
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

        let mut any_changed = false;
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
                    code.edit_range(start..end, new_definition);
                    any_changed = true;
                }
            }
        }
        // One incremental reparse for the whole batch, instead of one per
        // changed function.
        if any_changed {
            code.commit()?;
        }
        Ok(())
    }
}
