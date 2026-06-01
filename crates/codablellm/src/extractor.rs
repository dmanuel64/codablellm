use std::path::Path;

use crate::{function::Function, languages::Language, repo::Repo};

pub fn extract(repo: Repo) -> Vec<Function> {
    let mut functions = Vec::new();
    for source_file in repo.source_files() {
        if let Some(language) = Language::from_path(&source_file) {
            let source_file_functions = match language {
                Language::C => extract_c_file(&source_file),
                Language::Cpp => extract_cpp_file(&source_file),
                Language::Python => extract_python_file(&source_file),
                Language::JavaScript => extract_javascript_file(&source_file),
                Language::TypeScript => extract_typescript_file(&source_file),
                Language::Go => extract_go_file(&source_file),
                Language::Rust => extract_rust_file(&source_file),
                Language::Java => extract_java_file(&source_file),
                Language::CSharp => extract_csharp_file(&source_file),
            };
            functions.extend(source_file_functions);
        }
    }
    functions
}

fn extract_c_file(path: &Path) -> Vec<Function> {}

fn extract_cpp_file(path: &Path) -> Vec<Function> {}

fn extract_python_file(path: &Path) -> Vec<Function> {}

fn extract_javascript_file(path: &Path) -> Vec<Function> {}

fn extract_typescript_file(path: &Path) -> Vec<Function> {}

fn extract_go_file(path: &Path) -> Vec<Function> {}

fn extract_rust_file(path: &Path) -> Vec<Function> {}

fn extract_java_file(path: &Path) -> Vec<Function> {}

fn extract_csharp_file(path: &Path) -> Vec<Function> {}
