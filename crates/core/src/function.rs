use std::{
    borrow::Cow,
    fmt::Display,
    ops::Range,
    path::{Path, PathBuf},
    str::Utf8Error,
};

use indoc::indoc;
use serde::{Deserialize, Serialize};
use tree_sitter::StreamingIterator;

use crate::{
    SourceLanguage,
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
    pub definition: String,
    pub language: Language,
    pub location: Option<Location>,
}

pub trait Function {
    fn descriptor(&self) -> &Descriptor;

    fn name(&self) -> &str {
        &self.descriptor().name
    }

    fn definition(&self) -> &str {
        &self.descriptor().definition
    }

    fn language(&self) -> Language {
        self.descriptor().language
    }

    fn location(&self) -> &Option<Location> {
        &self.descriptor().location
    }
}

impl Display for dyn Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = self
            .location()
            .as_ref()
            .map(Location::to_string)
            .unwrap_or_else(|| String::from("<MEM>"));
        let func = self.name();
        write!(f, "{func} ({location})")
    }
}

pub trait ScopedFunction: Function {
    fn scope(&self) -> &[String];
}

impl Display for dyn ScopedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = (self as &dyn Function).to_string();
        let separator = match self.language() {
            Language::Source(SourceLanguage::Cpp | SourceLanguage::Rust) => "::",
            _ => ".",
        };
        let scope = self.scope().join(separator);
        write!(f, "{scope}{separator}{out}")
    }
}

pub trait Method: ScopedFunction {
    fn receiver(&self) -> &str;
}

impl Display for dyn Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = ToString::to_string(self as &dyn Function);
        let separator = match self.language() {
            Language::Source(SourceLanguage::Cpp | SourceLanguage::Rust) => "::",
            _ => ".",
        };
        let scope = self.scope().join(separator);
        let receiver = self.receiver();
        write!(f, "{scope}{separator}{receiver}{separator}{out}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceFunction {
    CFunction(CFunction),
    CppFunction(CppFunction),
    CppMethod(CppMethod),
    CppAssociatedFunction(CppAssociatedFunction),
    PythonFunction(PythonFunction),
    PythonMethod(PythonMethod),
    PythonAssociatedFunction(PythonAssociatedFunction),
    JavaScriptFunction(JavaScriptFunction),
    JavaScriptMethod(JavaScriptMethod),
    JavaScriptAssociatedFunction(JavaScriptAssociatedFunction),
    TypeScriptFunction(TypeScriptFunction),
    TypeScriptMethod(TypeScriptMethod),
    TypeScriptAssociatedFunction(TypeScriptAssociatedFunction),
    GoFunction(GoFunction),
    GoMethod(GoMethod),
    RustFunction(RustFunction),
    RustMethod(RustMethod),
    RustAssociatedFunction(RustAssociatedFunction),
    JavaMethod(JavaMethod),
    JavaAssociatedFunction(JavaAssociatedFunction),
    CSharpMethod(CSharpMethod),
    CSharpAssociatedFunction(CSharpAssociatedFunction),
}

impl Function for SourceFunction {
    fn descriptor(&self) -> &Descriptor {
        match self {
            SourceFunction::CFunction(func) => func.descriptor(),
            SourceFunction::CppFunction(func) => func.descriptor(),
            SourceFunction::CppMethod(func) => func.descriptor(),
            SourceFunction::CppAssociatedFunction(func) => func.descriptor(),
            SourceFunction::PythonFunction(func) => func.descriptor(),
            SourceFunction::PythonMethod(func) => func.descriptor(),
            SourceFunction::PythonAssociatedFunction(func) => func.descriptor(),
            SourceFunction::JavaScriptFunction(func) => func.descriptor(),
            SourceFunction::JavaScriptMethod(func) => func.descriptor(),
            SourceFunction::JavaScriptAssociatedFunction(func) => func.descriptor(),
            SourceFunction::TypeScriptFunction(func) => func.descriptor(),
            SourceFunction::TypeScriptMethod(func) => func.descriptor(),
            SourceFunction::TypeScriptAssociatedFunction(func) => func.descriptor(),
            SourceFunction::GoFunction(func) => func.descriptor(),
            SourceFunction::GoMethod(func) => func.descriptor(),
            SourceFunction::RustFunction(func) => func.descriptor(),
            SourceFunction::RustMethod(func) => func.descriptor(),
            SourceFunction::RustAssociatedFunction(func) => func.descriptor(),
            SourceFunction::JavaMethod(func) => func.descriptor(),
            SourceFunction::JavaAssociatedFunction(func) => func.descriptor(),
            SourceFunction::CSharpMethod(func) => func.descriptor(),
            SourceFunction::CSharpAssociatedFunction(func) => func.descriptor(),
        }
    }
}

impl SourceFunction {
    pub fn as_c_function(&self) -> Option<&CFunction> {
        if let SourceFunction::CFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_cpp_function(&self) -> Option<&CppFunction> {
        if let SourceFunction::CppFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_cpp_method(&self) -> Option<&CppMethod> {
        if let SourceFunction::CppMethod(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_cpp_associated_function(&self) -> Option<&CppAssociatedFunction> {
        if let SourceFunction::CppAssociatedFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_python_function(&self) -> Option<&PythonFunction> {
        if let SourceFunction::PythonFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_python_method(&self) -> Option<&PythonMethod> {
        if let SourceFunction::PythonMethod(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_python_associated_function(&self) -> Option<&PythonAssociatedFunction> {
        if let SourceFunction::PythonAssociatedFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_java_script_function(&self) -> Option<&JavaScriptFunction> {
        if let SourceFunction::JavaScriptFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_java_script_method(&self) -> Option<&JavaScriptMethod> {
        if let SourceFunction::JavaScriptMethod(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_java_script_associated_function(&self) -> Option<&JavaScriptAssociatedFunction> {
        if let SourceFunction::JavaScriptAssociatedFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_type_script_function(&self) -> Option<&TypeScriptFunction> {
        if let SourceFunction::TypeScriptFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_type_script_method(&self) -> Option<&TypeScriptMethod> {
        if let SourceFunction::TypeScriptMethod(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_type_script_associated_function(&self) -> Option<&TypeScriptAssociatedFunction> {
        if let SourceFunction::TypeScriptAssociatedFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_go_function(&self) -> Option<&GoFunction> {
        if let SourceFunction::GoFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_go_method(&self) -> Option<&GoMethod> {
        if let SourceFunction::GoMethod(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_rust_function(&self) -> Option<&RustFunction> {
        if let SourceFunction::RustFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_rust_method(&self) -> Option<&RustMethod> {
        if let SourceFunction::RustMethod(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_rust_associated_function(&self) -> Option<&RustAssociatedFunction> {
        if let SourceFunction::RustAssociatedFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_java_method(&self) -> Option<&JavaMethod> {
        if let SourceFunction::JavaMethod(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_java_associated_function(&self) -> Option<&JavaAssociatedFunction> {
        if let SourceFunction::JavaAssociatedFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_csharp_method(&self) -> Option<&CSharpMethod> {
        if let SourceFunction::CSharpMethod(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_csharp_associated_function(&self) -> Option<&CSharpAssociatedFunction> {
        if let SourceFunction::CSharpAssociatedFunction(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn scope(&self) -> Option<&[String]> {
        match self {
            SourceFunction::CFunction(_)
            | SourceFunction::CppFunction(_)
            | SourceFunction::PythonFunction(_)
            | SourceFunction::JavaScriptFunction(_)
            | SourceFunction::TypeScriptFunction(_)
            | SourceFunction::GoFunction(_)
            | SourceFunction::RustFunction(_) => None,
            SourceFunction::CppMethod(func) => Some(func.scope()),
            SourceFunction::CppAssociatedFunction(func) => Some(func.scope()),
            SourceFunction::PythonMethod(func) => Some(func.scope()),
            SourceFunction::PythonAssociatedFunction(func) => Some(func.scope()),
            SourceFunction::JavaScriptMethod(func) => Some(func.scope()),
            SourceFunction::JavaScriptAssociatedFunction(func) => Some(func.scope()),
            SourceFunction::TypeScriptMethod(func) => Some(func.scope()),
            SourceFunction::TypeScriptAssociatedFunction(func) => Some(func.scope()),
            SourceFunction::GoMethod(func) => Some(func.scope()),
            SourceFunction::RustMethod(func) => Some(func.scope()),
            SourceFunction::RustAssociatedFunction(func) => Some(func.scope()),
            SourceFunction::JavaMethod(func) => Some(func.scope()),
            SourceFunction::JavaAssociatedFunction(func) => Some(func.scope()),
            SourceFunction::CSharpMethod(func) => Some(func.scope()),
            SourceFunction::CSharpAssociatedFunction(func) => Some(func.scope()),
        }
    }

    pub fn receiver(&self) -> Option<&str> {
        match self {
            SourceFunction::CFunction(_)
            | SourceFunction::CppFunction(_)
            | SourceFunction::PythonFunction(_)
            | SourceFunction::JavaScriptFunction(_)
            | SourceFunction::TypeScriptFunction(_)
            | SourceFunction::GoFunction(_)
            | SourceFunction::RustFunction(_)
            | SourceFunction::CppAssociatedFunction(_)
            | SourceFunction::PythonAssociatedFunction(_)
            | SourceFunction::JavaScriptAssociatedFunction(_)
            | SourceFunction::TypeScriptAssociatedFunction(_)
            | SourceFunction::RustAssociatedFunction(_)
            | SourceFunction::JavaAssociatedFunction(_)
            | SourceFunction::CSharpAssociatedFunction(_) => None,
            SourceFunction::CppMethod(func) => Some(func.receiver()),
            SourceFunction::PythonMethod(func) => Some(func.receiver()),
            SourceFunction::JavaScriptMethod(func) => Some(func.receiver()),
            SourceFunction::TypeScriptMethod(func) => Some(func.receiver()),
            SourceFunction::GoMethod(func) => Some(func.receiver()),
            SourceFunction::RustMethod(func) => Some(func.receiver()),
            SourceFunction::JavaMethod(func) => Some(func.receiver()),
            SourceFunction::CSharpMethod(func) => Some(func.receiver()),
        }
    }
}

impl Display for SourceFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceFunction::CFunction(func) => write!(f, "{func}"),
            SourceFunction::CppFunction(func) => write!(f, "{func}"),
            SourceFunction::CppMethod(func) => write!(f, "{func}"),
            SourceFunction::CppAssociatedFunction(func) => write!(f, "{func}"),
            SourceFunction::PythonFunction(func) => write!(f, "{func}"),
            SourceFunction::PythonMethod(func) => write!(f, "{func}"),
            SourceFunction::PythonAssociatedFunction(func) => write!(f, "{func}"),
            SourceFunction::JavaScriptFunction(func) => write!(f, "{func}"),
            SourceFunction::JavaScriptMethod(func) => write!(f, "{func}"),
            SourceFunction::JavaScriptAssociatedFunction(func) => write!(f, "{func}"),
            SourceFunction::TypeScriptFunction(func) => write!(f, "{func}"),
            SourceFunction::TypeScriptMethod(func) => write!(f, "{func}"),
            SourceFunction::TypeScriptAssociatedFunction(func) => write!(f, "{func}"),
            SourceFunction::GoFunction(func) => write!(f, "{func}"),
            SourceFunction::GoMethod(func) => write!(f, "{func}"),
            SourceFunction::RustFunction(func) => write!(f, "{func}"),
            SourceFunction::RustMethod(func) => write!(f, "{func}"),
            SourceFunction::RustAssociatedFunction(func) => write!(f, "{func}"),
            SourceFunction::JavaMethod(func) => write!(f, "{func}"),
            SourceFunction::JavaAssociatedFunction(func) => write!(f, "{func}"),
            SourceFunction::CSharpMethod(func) => write!(f, "{func}"),
            SourceFunction::CSharpAssociatedFunction(func) => write!(f, "{func}"),
        }
    }
}

mod c;
mod cpp;
mod csharp;
mod go;
mod java;
mod javascript;
mod python;
mod rust;
mod typescript;

pub use c::CFunction;
pub use cpp::{CppAssociatedFunction, CppFunction, CppMethod};
pub use csharp::{CSharpAssociatedFunction, CSharpMethod};
pub use go::{GoFunction, GoMethod};
pub use java::{JavaAssociatedFunction, JavaMethod};
pub use javascript::{JavaScriptAssociatedFunction, JavaScriptFunction, JavaScriptMethod};
pub use python::{PythonAssociatedFunction, PythonFunction, PythonMethod};
pub use rust::{RustAssociatedFunction, RustFunction, RustMethod};
pub use typescript::{TypeScriptAssociatedFunction, TypeScriptFunction, TypeScriptMethod};

// pub struct ByteCodeFunction;
pub struct AssemblyFunction;
pub struct DecompiledFunction;

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
