use std::{
    cell::RefCell,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    str::Utf8Error,
};

use thiserror::Error as ThisError;

use crate::Language;

thread_local! {
    static PARSER: RefCell<tree_sitter::Parser> = RefCell::new({
        tree_sitter::Parser::new()
    });
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("failed to parse source code: {}",
    path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "<UNKNOWN>".into()))]
    Parse {
        path: Option<PathBuf>,
        #[source]
        source: Option<io::Error>,
    },
    #[error("failed to decode source code")]
    Decode(#[from] Utf8Error),
    #[error("Failed to recognize language from source code file: {}",
    file.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "<UNKNOWN>".into()))]
    UnknownLanguage { file: PathBuf },
    #[error("failed to query S-expression")]
    Query(#[source] tree_sitter::QueryError),
}

pub struct ParsedCode {
    tree: tree_sitter::Tree,
    language: Language,
    code: Vec<u8>,
    pub source: Option<PathBuf>,
    query: Option<tree_sitter::Query>,
    cursor: tree_sitter::QueryCursor,
}

pub fn parse(language: Language, text: impl Into<Vec<u8>>) -> Result<ParsedCode, Error> {
    ParsedCode::new(language, text, None)
}

pub fn parse_file(file: &Path) -> Result<ParsedCode, Error> {
    ParsedCode::try_from(file)
}

impl ParsedCode {
    pub fn new(
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
        })
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    /// Compiles `sexp` and runs it against the parsed tree, returning both
    /// the compiled query (so callers can look up capture indices) and the
    /// resulting matches. Both borrow `self` for `'a`, so no further access
    /// to `self` is possible until the caller is done with them - grab
    /// anything else you need from `self` before calling this.
    pub fn query<'a>(
        &'a mut self,
        sexp: &str,
    ) -> Result<
        (
            &'a tree_sitter::Query,
            tree_sitter::QueryMatches<'a, 'a, &'a [u8], &'a [u8]>,
        ),
        Error,
    > {
        let root_node = self.tree.root_node();
        let compiled = tree_sitter::Query::new(&root_node.language(), sexp).map_err(Error::Query)?;
        self.query = Some(compiled);
        let Self { query, cursor, code, .. } = self;
        let query_ref = query.as_ref().expect("query was just set");
        let matches = cursor.matches(query_ref, root_node, code.as_slice());
        Ok((query_ref, matches))
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

impl ParsedCode {
    /// Like `TryFrom<&Path>`, but resolves ambiguous `.h` files as C++
    /// instead of C when `headers_as_cpp` is set (`Language::from_path`
    /// always resolves `.h` to C, since C is declared before C++ and both
    /// languages claim that extension).
    pub fn try_from_path_with_options(value: &Path, headers_as_cpp: bool) -> Result<Self, Error> {
        let Some(language) = Language::from_path(value) else {
            return Err(Error::UnknownLanguage {
                file: value.to_path_buf(),
            });
        };
        let is_header = value
            .extension()
            .map(|ext| ext.eq_ignore_ascii_case("h"))
            .unwrap_or(false);
        let language = if headers_as_cpp && language == Language::C && is_header {
            Language::Cpp
        } else {
            language
        };
        let text = fs::read(value).map_err(|e| Error::Parse {
            path: Some(value.to_path_buf()),
            source: Some(e),
        })?;
        Self::new(language, text, Some(value.to_path_buf()))
    }
}

impl TryFrom<&Path> for ParsedCode {
    type Error = Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Self::try_from_path_with_options(value, false)
    }
}
