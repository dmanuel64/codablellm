use indicatif::{HumanCount, ParallelProgressIterator};
use rayon::prelude::*;
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    str::Utf8Error,
};
use thiserror::Error;
use tree_sitter::StreamingIterator;

use crate::{
    ProgressDisplay,
    function::{self, Function},
    language::Language,
    repo::Repository,
};

thread_local! {
    static PARSER: RefCell<tree_sitter::Parser> = RefCell::new({
        tree_sitter::Parser::new()
    });
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse source code: {}",
    path.as_ref().map(|p| p.as_display().to_string()).unwrap_or_else(|| "<UNKNOWN>".into()))]
    Parse {
        path: Option<PathBuf>,
        #[source]
        source: Option<io::Error>,
    },
    #[error("failed to decode source code")]
    Decode(#[from] Utf8Error),
    #[error("failed to transform function '{}': {source}", function.name())]
    Transform {
        function: Function,
        source: anyhow::Error,
    },
    #[error("Failed to recognize language from source code file: {}",
    file.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "<UNKNOWN>".into()))]
    UnknownLanguage { file: PathBuf },
    #[error("failed to query S-expression")]
    Query(#[source] tree_sitter::QueryError),
}

pub enum Transform {
    Native(Box<dyn for<'a> Fn(&'a Function) -> anyhow::Result<MaybeChangedFunction<'a>> + Send>),
    #[cfg(feature = "rhai")]
    Rhai {
        file: PathBuf,
    },
}

pub type MaybeChangedFunction<'a> = Cow<'a, Function>;

pub trait MaybeChangedFunctionExt {
    fn is_changed(&self) -> bool;
}

impl MaybeChangedFunctionExt for MaybeChangedFunction<'_> {
    fn is_changed(&self) -> bool {
        matches!(self, Cow::Owned(_))
    }
}

impl Transform {
    pub fn apply<'a>(&self, function: &'a Function) -> Result<MaybeChangedFunction<'a>, Error> {
        match self {
            Transform::Native(f) => f(function),
            #[cfg(feature = "rhai")]
            Transform::Rhai { file } => {
                let mut engine = rhai::Engine::new();
                engine.register_type::<Function>();
                let mut scope = rhai::Scope::new();
                scope.push("function", function.clone());
                engine
                    .run_file_with_scope(&mut scope, file.clone())
                    .map(|_| {
                        let new_function = scope
                            .get_value_ref::<Function>("function")
                            .unwrap_or(function);
                        if new_function.definition() == function.definition() {
                            Cow::Borrowed(function)
                        } else {
                            Cow::Owned(new_function.clone())
                        }
                    })
                    .map_err(anyhow::Error::from)
            }
        }
        .map_err(|source| Error::Transform {
            function: function.clone(),
            source,
        })
    }
}

#[derive(Debug, Default)]
pub struct Options {
    pub progress_display: ProgressDisplay,
    pub headers_as_cpp: bool,
}

pub fn extract(repo: &Repository) -> Result<Vec<Function>, Error> {
    extract_with_options(repo, &Options::default())
}

pub fn extract_with_options(repo: &Repository, options: &Options) -> Result<Vec<Function>, Error> {
    extract_inner(repo, None, options)
}

pub fn transform(repo: &Repository, transform: &Transform) -> Result<Vec<Function>, Error> {
    transform_with_options(repo, transform, &Options::default())
}

pub fn transform_with_options(
    repo: &Repository,
    transform: &Transform,
    options: &Options,
) -> Result<Vec<Function>, Error> {
    extract_inner(repo, Some(transform), options)
}

fn extract_inner(
    repo: &Repository,
    transform: Option<&Transform>,
    Options {
        progress_display,
        headers_as_cpp,
    }: &Options,
) -> Result<Vec<Function>, Error> {
    let progress = progress_display.new_progress_bar(None);
    let source_files = repo.source_files();
    if let (_, Some(num_files)) = source_files.size_hint() {
        tracing::info!(
            "Will attempt to extract {} source code files",
            HumanCount(num_files as u64)
        )
    }
    repo.source_files()
        .par_bridge()
        .progress_with(progress)
        .map(|path| {
            let mut functions = Vec::new();
            tracing::debug!(file = %path.display(), "Extracting source code file");
            match extract_file(&path, *headers_as_cpp) {
                Ok(f) => functions.extend(f),
                Err(error) => {
                    if let Error::UnknownLanguage { file } = error {
                        tracing::warn!(
                            file = %file.display(),
                            "Failed to recognize language from file"
                        );
                    } else {
                        tracing::warn!(?error, file = %path.display(), "Failed to extract source code functions");
                    }
                }
            }
            Ok(functions)
        }).flatten().collect()
}

fn extract_file(path: &Path, headers_as_cpp: bool) -> Result<Vec<Function>, Error> {
    let code = ParsedCode::try_from(path)?;
}

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

struct ParsedCode {
    tree: tree_sitter::Tree,
    language: Language,
    code: Vec<u8>,
    pub source: Option<PathBuf>,
    query: Option<tree_sitter::Query>,
    cursor: tree_sitter::QueryCursor,
    functions: Vec<Function>,
}

impl ParsedCode {
    pub fn parse(
        language: Language,
        text: impl Into<Vec<u8>>,
        source: Option<PathBuf>,
    ) -> Result<Self, Error> {
        let code = text.into();
        let tree = PARSER.with_borrow_mut(|parser| {
            parser
                .set_language(&if let Language::TypeScript = language
                    && source
                        .as_ref()
                        .map(PathBuf::as_path)
                        .and_then(Path::extension)
                        .map(OsStr::to_string_lossy)
                        .map(|ext| ext.eq_ignore_ascii_case("tsx"))
                        .unwrap_or_default()
                {
                    tree_sitter_typescript::LANGUAGE_TSX.into()
                } else {
                    language.into()
                })
                .expect("the language to be set correctly for the parser");
            parser.parse(&code, None).ok_or_else(|| Error::Parse {
                path: None,
                source: None,
            })
        })?;
        Ok(Self {
            tree,
            language,
            code,
            source,
            query: None,
            cursor: tree_sitter::QueryCursor::new(),
            functions: Vec::new(),
        })
    }

    pub fn query<'a>(
        &'a mut self,
        sexp: &str,
    ) -> Result<tree_sitter::QueryMatches<'a, 'a, &[u8], &[u8]>, Error> {
        let root_node = self.tree.root_node();
        let compiled =
            tree_sitter::Query::new(&root_node.language(), sexp).map_err(|e| Error::Query(e))?;
        self.query = Some(compiled);
        Ok(self.cursor.matches(
            self.query.as_ref().unwrap(),
            root_node,
            self.code.as_slice(),
        ))
    }

    fn functions_inner(&mut self) -> Result<Vec<Function>, Error> {
        let sexp = match self.language {
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

        let language = self.language;
        let source = self.source.clone();
        let code = self.code.clone();

        let name_idx = self
            .query
            .as_ref()
            .expect("query to be populated")
            .capture_index_for_name("name")
            .expect("The s-expression to contain the name capture group");
        let definition_idx = self
            .query
            .as_ref()
            .expect("query to be populated")
            .capture_index_for_name("definition")
            .expect("The s-expression to contain the definition capture group");
        let mut matches = self.query(sexp)?;
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

    fn functions_mut(&mut self) -> Result<&mut Vec<Function>, Error> {
        if self.functions.is_empty() {
            self.functions = self.functions_inner()?;
        }
        Ok(&mut self.functions)
    }

    pub fn edit<EditFn>(&mut self, e: EditFn) -> Result<(), Error>
    where
        EditFn: FnOnce(&mut Vec<u8>),
    {
        let old_code = self.code.clone();
        e(&mut self.code);
        let new_code = &self.code;

        let common_prefix = old_code
            .iter()
            .zip(new_code.iter())
            .take_while(|(a, b)| a == b)
            .count();
        let old_suffix_max = old_code.len() - common_prefix;
        let new_suffix_max = new_code.len() - common_prefix;
        let common_suffix = old_code[common_prefix..]
            .iter()
            .rev()
            .zip(new_code[common_prefix..].iter().rev())
            .take(old_suffix_max.min(new_suffix_max))
            .take_while(|(a, b)| a == b)
            .count();

        let start_byte = common_prefix;
        let old_end_byte = old_code.len() - common_suffix;
        let new_end_byte = new_code.len() - common_suffix;

        self.tree.edit(&tree_sitter::InputEdit {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_position: point_at(&old_code, start_byte),
            old_end_position: point_at(&old_code, old_end_byte),
            new_end_position: point_at(new_code, new_end_byte),
        });

        self.tree = PARSER.with_borrow_mut(|parser| {
            parser
                .set_language(&self.language.into())
                .expect("the language to be set correctly for the parser");
            parser
                .parse(&self.code, Some(&self.tree))
                .ok_or_else(|| Error::Parse {
                    path: self.source.clone(),
                    source: None,
                })
        })?;
        Ok(())
    }

    pub fn edit_function(
        &mut self,
        old_function: &Function,
        new_function: Function,
    ) -> Result<(), Error> {
        if let Some(f) = self
            .functions_mut()?
            .iter_mut()
            .find(|f| *f == old_function)
        {
            self.edit(|bytes| {});
            *f = new_function;
            Ok(())
        } else {
            todo!("custom error")
        }
    }
}

fn point_at(bytes: &[u8], byte_offset: usize) -> tree_sitter::Point {
    let mut row = 0;
    let mut column = 0;
    for &b in &bytes[..byte_offset] {
        if b == b'\n' {
            row += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
    tree_sitter::Point { row, column }
}

impl TryFrom<&Path> for ParsedCode {
    type Error = Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        if let Some(language) = Language::from_path(&value) {
            let text = fs::read(value).map_err(|e| Error::Parse {
                path: Some(value.to_path_buf()),
                source: Some(e),
            })?;
            Self::parse(language, text, Some(value.to_path_buf()))
        } else {
            Err(Error::UnknownLanguage {
                file: value.to_path_buf(),
            })
        }
    }
}
