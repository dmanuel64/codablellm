use std::{
    fs,
    path::{Path, PathBuf},
};

use thiserror::Error;
use tree_sitter::StreamingIterator;

use crate::{function::Function, language::Language, repo::Repo};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse {path}")]
    Parse { path: PathBuf },
    #[error("failed to decode")]
    Decode,
}

#[derive(Debug)]
pub struct Options {
    pub display_progress: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            display_progress: true,
        }
    }
}

pub fn extract(repo: &Repo) -> Result<Vec<Function>, Error> {
    extract_with_options(repo, &Options::default())
}

pub fn extract_with_options(repo: &Repo, options: &Options) -> Result<Vec<Function>, Error> {
    let mut functions = Vec::new();
    let mut parser = tree_sitter::Parser::new();
    for source_file in repo.source_files() {
        if let Some(language) = Language::from_path(&source_file) {
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
                Err(_) => todo!(),
            };
        }
    }
    Ok(functions)
}

fn query_functions(
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
                .map_err(|_| Error::Decode)?
                .to_string();
            let definition = def
                .node
                .utf8_text(source)
                .map_err(|_| Error::Decode)?
                .to_string();
            functions.push(Function { name, definition });
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
    let source = fs::read(path).map_err(|_| Error::Parse {
        path: path.to_path_buf(),
    })?;
    let tree = parser.parse(&source, None).ok_or_else(|| Error::Parse {
        path: path.to_path_buf(),
    })?;
    Ok((tree, source))
}

fn extract_c_file(parser: &mut tree_sitter::Parser, path: &Path) -> Result<Vec<Function>, Error> {
    let (tree, source) = parse(parser, &tree_sitter_c::LANGUAGE.into(), path)?;
    query_functions(
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
        &source,
        tree.root_node(),
        r#"
        (method_declaration
            name: (identifier) @name
        ) @definition
    "#,
    )
}
