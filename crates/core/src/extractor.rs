use indicatif::HumanCount;
use std::{
    fs, io,
    path::{Path, PathBuf},
    str::Utf8Error,
};
use thiserror::Error;
use tree_sitter::StreamingIterator;

use crate::{ProgressDisplay, function::Function, language::Language, repo::Repository};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse {path}")]
    Parse {
        path: PathBuf,
        #[source]
        source: Option<io::Error>,
    },
    #[error("failed to decode")]
    Decode(#[from] Utf8Error),
    #[error("failed to transform function '{}': {source}", function.name())]
    Transform {
        function: Function,
        source: anyhow::Error,
    },
}

pub enum Transform {
    Native(Box<dyn Fn(&Function) -> anyhow::Result<String> + Send>),
    #[cfg(feature = "rhai")]
    Rhai {
        file: PathBuf,
    },
}

impl Transform {
    pub fn eval(&self, function: &Function) -> Result<String, Error> {
        match self {
            Transform::Native(f) => f(function),
            #[cfg(feature = "rhai")]
            Transform::Rhai { file } => {
                let engine = rhai::Engine::new();
                engine.eval_file(file.clone()).map_err(anyhow::Error::from)
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
    Options { progress_display }: &Options,
) -> Result<Vec<Function>, Error> {
    let mut functions = Vec::new();
    let progress = progress_display.new_progress_bar(None);
    let mut parser = tree_sitter::Parser::new();
    let source_files = progress.wrap_iter(repo.source_files().into_iter());
    if let (_, Some(num_files)) = source_files.size_hint() {
        tracing::info!(
            "Will attempt to extract {} source code files",
            HumanCount(num_files as u64)
        )
    }
    for source_file in source_files {
        if let Some(language) = Language::from_path(&source_file) {
            tracing::debug!(file = %source_file.display(), "Extracting source code file");
            let extraction_results = match language {
                Language::C => extract_c_file(&mut parser, &source_file),
                Language::Cpp => extract_cpp_file(&mut parser, &source_file),
                Language::Python => extract_python_file(&mut parser, &source_file),
                Language::JavaScript => extract_javascript_file(&mut parser, &source_file),
                Language::TypeScript => extract_typescript_file(&mut parser, &source_file),
                Language::Go => extract_go_file(&mut parser, &source_file),
                Language::Rust => extract_rust_file(&mut parser, &source_file),
                Language::Java => extract_java_file(&mut parser, &source_file),
                Language::CSharp => extract_csharp_file(&mut parser, &source_file),
            };
            match extraction_results {
                Ok(source_file_functions) => functions.extend(source_file_functions),
                Err(error) => {
                    tracing::warn!(?error, file = %source_file.display(), "Failed to extract source code functions");
                }
            };
        } else {
            tracing::warn!(
                file = %source_file.display(),
                "Failed to recognize language from file"
            );
        }
    }
    Ok(functions)
}

fn query_functions(
    file: PathBuf,
    language: Language,
    source: &[u8],
    node: tree_sitter::Node,
    sexp: &str,
) -> Result<Vec<Function>, Error> {
    let mut functions = Vec::new();
    let query =
        tree_sitter::Query::new(&node.language(), sexp).expect("the s-expression to be valid");
    let mut cursor = tree_sitter::QueryCursor::new();
    let name_idx = query
        .capture_index_for_name("name")
        .expect("The s-expression to contain the name capture group");
    let definition_idx = query
        .capture_index_for_name("definition")
        .expect("The s-expression to contain the definition capture group");
    let mut matches = cursor.matches(&query, node, source);
    while let Some(m) = matches.next() {
        let name_capture = m.captures.iter().find(|c| c.index == name_idx);
        let definition_capture = m.captures.iter().find(|c| c.index == definition_idx);
        if let (Some(name), Some(def)) = (name_capture, definition_capture) {
            let name = name
                .node
                .utf8_text(source)
                .map_err(Utf8Error::from)?
                .to_string();
            let definition = def
                .node
                .utf8_text(source)
                .map_err(Utf8Error::from)?
                .to_string();
            let range = def.node.range();
            let line_range = range.start_point.row..range.end_point.row;
            let column_range = range.start_point.column..range.end_point.column;
            functions.push(Function::Source {
                name,
                definition,
                file: file.clone(),
                language,
                line_range,
                column_range,
            });
        }
    }
    Ok(functions)
}

fn parse(
    parser: &mut tree_sitter::Parser,
    language: &tree_sitter::Language,
    path: &Path,
) -> Result<(tree_sitter::Tree, Vec<u8>), Error> {
    parser
        .set_language(&language)
        .expect("the language to be set correctly for the parser");
    let source = fs::read(path).map_err(|e| Error::Parse {
        path: path.to_path_buf(),
        source: Some(e),
    })?;
    let tree = parser.parse(&source, None).ok_or_else(|| Error::Parse {
        path: path.to_path_buf(),
        source: None,
    })?;
    Ok((tree, source))
}

fn extract_c_file(parser: &mut tree_sitter::Parser, path: &Path) -> Result<Vec<Function>, Error> {
    let (tree, source) = parse(parser, &tree_sitter_c::LANGUAGE.into(), path)?;
    query_functions(
        path.to_path_buf(),
        Language::C,
        &source,
        tree.root_node(),
        r#"
        (function_definition
            declarator: (function_declarator
                declarator: (identifier) @name)
        ) @definition
    "#,
    )
}

fn extract_cpp_file(parser: &mut tree_sitter::Parser, path: &Path) -> Result<Vec<Function>, Error> {
    let (tree, source) = parse(parser, &tree_sitter_cpp::LANGUAGE.into(), path)?;
    query_functions(
        path.to_path_buf(),
        Language::Cpp,
        &source,
        tree.root_node(),
        r#"
        (function_definition
            declarator: (function_declarator
                declarator: [(identifier) (field_identifier)] @name)
        ) @definition
    "#,
    )
}

fn extract_python_file(
    parser: &mut tree_sitter::Parser,
    path: &Path,
) -> Result<Vec<Function>, Error> {
    let (tree, source) = parse(parser, &tree_sitter_python::LANGUAGE.into(), path)?;
    query_functions(
        path.to_path_buf(),
        Language::Python,
        &source,
        tree.root_node(),
        r#"
        (function_definition
            name: (identifier) @name
        ) @definition
    "#,
    )
}

fn extract_javascript_file(
    parser: &mut tree_sitter::Parser,
    path: &Path,
) -> Result<Vec<Function>, Error> {
    let (tree, source) = parse(parser, &tree_sitter_javascript::LANGUAGE.into(), path)?;
    query_functions(
        path.to_path_buf(),
        Language::JavaScript,
        &source,
        tree.root_node(),
        r#"
        [
            (function_declaration
                name: (identifier) @name) @definition
            (method_definition
                name: (property_identifier) @name) @definition
        ]
    "#,
    )
}

fn extract_typescript_file(
    parser: &mut tree_sitter::Parser,
    path: &Path,
) -> Result<Vec<Function>, Error> {
    let language = if path.extension().unwrap_or_default().to_string_lossy() == "tsx" {
        tree_sitter_typescript::LANGUAGE_TSX
    } else {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT
    };
    let (tree, source) = parse(parser, &language.into(), path)?;
    query_functions(
        path.to_path_buf(),
        Language::TypeScript,
        &source,
        tree.root_node(),
        r#"
        [
            (function_declaration
                name: (identifier) @name) @definition
            (method_definition
                name: (property_identifier) @name) @definition
        ]
    "#,
    )
}

fn extract_go_file(parser: &mut tree_sitter::Parser, path: &Path) -> Result<Vec<Function>, Error> {
    let (tree, source) = parse(parser, &tree_sitter_go::LANGUAGE.into(), path)?;
    query_functions(
        path.to_path_buf(),
        Language::Go,
        &source,
        tree.root_node(),
        r#"
        (function_declaration
            name: (identifier) @name
        ) @definition
        (method_declaration
            name: (field_identifier) @name
        ) @definition
    "#,
    )
}

fn extract_rust_file(
    parser: &mut tree_sitter::Parser,
    path: &Path,
) -> Result<Vec<Function>, Error> {
    let (tree, source) = parse(parser, &tree_sitter_rust::LANGUAGE.into(), path)?;
    query_functions(
        path.to_path_buf(),
        Language::Rust,
        &source,
        tree.root_node(),
        r#"
        (function_item
            name: (identifier) @name
        ) @definition
    "#,
    )
}

fn extract_java_file(
    parser: &mut tree_sitter::Parser,
    path: &Path,
) -> Result<Vec<Function>, Error> {
    let (tree, source) = parse(parser, &tree_sitter_java::LANGUAGE.into(), path)?;
    query_functions(
        path.to_path_buf(),
        Language::Java,
        &source,
        tree.root_node(),
        r#"
        (method_declaration
            name: (identifier) @name
        ) @definition
    "#,
    )
}

fn extract_csharp_file(
    parser: &mut tree_sitter::Parser,
    path: &Path,
) -> Result<Vec<Function>, Error> {
    let (tree, source) = parse(parser, &tree_sitter_c_sharp::LANGUAGE.into(), path)?;
    query_functions(
        path.to_path_buf(),
        Language::CSharp,
        &source,
        tree.root_node(),
        r#"
        (method_declaration
            name: (identifier) @name
        ) @definition
    "#,
    )
}
