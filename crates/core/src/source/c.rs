#![cfg(feature = "c")]

use std::{ops::Range, sync::Arc};

use thiserror::Error;
use treesitter_types_c::{
    tree_sitter::{Query, QueryCursor, StreamingIterator},
    *,
};

use crate::source::{Language, ParsedCode, Subroutine, subroutine::Function};

const SUBROUTINE_SEXPR: &str = include_str!("../../assets/queries/c.scm");
#[derive(Debug, Clone)]
pub struct C;

#[derive(Debug, Clone)]
pub struct Extra {
    return_type_span: Range<usize>,
}

#[derive(Debug, Error, Clone)]
pub enum ParseError {}

impl Language for C {
    const NAME: &str = "C";
    const FILE_EXTENSIONS: &[&str] = &[".c", ".h"];

    type Tree = tree_sitter::Tree;
    type ParseError = ParseError;
    type Extra = Extra;

    fn reparse(text: &str, old_tree: Option<&Self::Tree>) -> Result<Self::Tree, Self::ParseError> {
        let mut parser = tree_sitter::Parser::new();
        // TODO: eventually use parse_with_options to have a progress bar callback
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .expect("tree-sitter-c version mismatch");
        Ok(parser.parse(text, old_tree).expect("language is set"))
    }

    fn subroutines(
        code: &Arc<ParsedCode<Self>>,
    ) -> Vec<Result<Subroutine<Self>, Self::ParseError>> {
        let query =
            Query::new(&code.tree().language(), SUBROUTINE_SEXPR).expect("query to compile");
        let mut cursor = QueryCursor::new();
        // TODO: use matches with options to have a progress bar callback
        let text = code.text.to_string();
        let captures = cursor.captures(&query, code.tree().root_node(), text.as_bytes());
        let capture_groups = query.capture_names();
        captures
            .map(|(m, _)| {
                // TODO: optimize this with a zip
                let name_span = m.captures.iter().find_map(|capture| {
                    if capture_groups[capture.index as usize] == "function.name" {
                        Some(capture.node.byte_range())
                    } else {
                        None
                    }
                });
                let definition_span = m
                    .captures
                    .iter()
                    .find_map(|capture| {
                        if capture_groups[capture.index as usize] == "function.definition" {
                            Some(capture.node.byte_range())
                        } else {
                            None
                        }
                    })
                    .unwrap();
                let return_type_span = m
                    .captures
                    .iter()
                    .find_map(|capture| {
                        if capture_groups[capture.index as usize] == "function.return_type" {
                            Some(capture.node.byte_range())
                        } else {
                            None
                        }
                    })
                    .unwrap();
                Ok(Subroutine::<C>::new(
                    code.clone(),
                    name_span,
                    definition_span,
                    Extra { return_type_span },
                ))
            })
            .cloned()
            .collect()
    }
}

impl Function for C {
    fn return_type_span(subroutine: &Subroutine<Self>) -> Option<Range<usize>> {
        Some(subroutine.extra().return_type_span.clone())
    }
}
