#![cfg(feature = "c")]

use crate::source::Source;
use std::{ops::Range, sync::Arc};

use treesitter_types_c::*;

use crate::source::{Language, ParsedCode, Subroutine, subroutine::Function};

#[derive(Debug, Clone)]
pub struct C;

#[derive(Debug, Clone)]
pub struct Extra {
    return_type_span: Range<usize>,
}

impl Language for C {
    const NAME: &str = "C";
    const FILE_EXTENSIONS: &[&str] = &[".c", ".h"];

    type Tree = tree_sitter::Tree;
    type ParseError = treesitter_types_c::ParseError;
    type Extra = Extra;

    fn parse(text: &str) -> Result<Self::Tree, Self::ParseError> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .expect("tree-sitter-c version mismatch");
        Ok(parser.parse(text, None).expect("language is set"))
    }

    fn subroutines(
        code: &Arc<ParsedCode<Self>>,
    ) -> Vec<Result<Subroutine<Self>, Self::ParseError>> {
        let bytes = code.text.to_string();
        let unit = match TranslationUnit::from_node(code.tree().root_node(), bytes.as_bytes()) {
            Ok(unit) => unit,
            Err(e) => return vec![Err(e)],
        };

        unit.children
            .iter()
            .filter_map(|child| {
                let TranslationUnitChildren::FunctionDefinition(def) = child else {
                    return None;
                };
                let span = child.span();
                let name_span = todo!("span of the identifier in def's declarator");
                let extra = Extra {
                    return_type_span: todo!("span of def's type"),
                };
                Some(Ok(Subroutine::new(
                    code.clone(),
                    name_span,
                    span.start_byte..span.end_byte,
                    extra,
                )))
            })
            .collect()
    }
}

impl Function for C {
    fn return_type_span(subroutine: &Subroutine<Self>) -> Option<Range<usize>> {
        Some(subroutine.extra().return_type_span.clone())
    }
}
