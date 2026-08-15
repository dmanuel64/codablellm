use std::{
    cell::RefCell,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use crate::{Language, function::Function};

thread_local! {
    static PARSER: RefCell<tree_sitter::Parser> = RefCell::new({
        tree_sitter::Parser::new()
    });
}

pub struct ParsedCode {
    tree: tree_sitter::Tree,
    language: Language,
    code: Vec<u8>,
    pub source: Option<PathBuf>,
    pub(super) query: Option<tree_sitter::Query>,
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

    pub fn query<'a>(
        &'a mut self,
        sexp: &str,
    ) -> Result<tree_sitter::QueryMatches<'a, 'a, &[u8], &[u8]>, Error> {
        let root_node = self.tree.root_node();
        let compiled =
            tree_sitter::Query::new(&root_node.language(), sexp).map_err(|e| Error::Query(e))?;
        self.query = Some(compiled);
        Ok(self
            .cursor
            .matches(self.query.as_ref().unwrap(), root_node, self.code()))
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

impl TryFrom<&Path> for ParsedCode {
    type Error = Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        if let Some(language) = Language::from_path(&value) {
            let text = fs::read(value).map_err(|e| Error::Parse {
                path: Some(value.to_path_buf()),
                source: Some(e),
            })?;
            Self::new(language, text, Some(value.to_path_buf()))
        } else {
            Err(Error::UnknownLanguage {
                file: value.to_path_buf(),
            })
        }
    }
}
