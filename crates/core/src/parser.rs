use std::{
    cell::RefCell,
    ffi::OsStr,
    fs, io,
    ops::Range,
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

    /// Splices `replacement` into the byte range `range`, updating the
    /// tree's edit bookkeeping for that region. Doesn't reparse - call
    /// `commit()` once after all edits are staged, so N edits share a
    /// single incremental reparse instead of paying for one each.
    ///
    /// Unlike a generic "mutate then diff the whole buffer" approach, this
    /// goes straight from the known range to the `InputEdit`, since the
    /// caller already knows exactly what changed.
    pub fn edit_range(&mut self, range: Range<usize>, replacement: impl Into<Vec<u8>>) {
        let replacement = replacement.into();

        let start_byte = range.start;
        let old_end_byte = range.end;
        let new_end_byte = range.start + replacement.len();
        let start_position = point_at(&self.code, start_byte);
        let old_end_position = point_at(&self.code, old_end_byte);
        let new_end_position = advance_point(start_position, &replacement);

        self.code.splice(range, replacement);

        self.tree.edit(&tree_sitter::InputEdit {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_position,
            old_end_position,
            new_end_position,
        });
    }

    /// Reparses after one or more `edit_range` calls, incrementally reusing
    /// whatever those `tree.edit()` calls marked as unaffected. A no-op to
    /// call `edit_range` and never `commit()` other than leaving the tree
    /// out of sync with `code()` - always pair them.
    pub fn commit(&mut self) -> Result<(), Error> {
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

/// Walks `bytes` from `start`, tracking row/column, to find the position
/// where it ends - used for `new_end_position` since the replacement
/// hasn't been parsed into a tree yet.
fn advance_point(start: tree_sitter::Point, bytes: &[u8]) -> tree_sitter::Point {
    let mut point = start;
    for &b in bytes {
        if b == b'\n' {
            point.row += 1;
            point.column = 0;
        } else {
            point.column += 1;
        }
    }
    point
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
