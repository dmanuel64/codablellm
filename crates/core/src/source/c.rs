#![cfg(feature = "c")]

use std::ops::Range;

use treesitter_types_c::*;

use crate::source::{Language, Tree, subroutine::Function};

#[derive(Debug, Clone)]
pub struct C;

impl Language for C {
    const NAME: &str = "C";

    const FILE_EXTENSIONS: &[&str] = &[".c", ".h"];

    type Tree = treesitter_types_c::tree_sitter::Tree;

    type ParseError = treesitter_types_c::ParseError;

    type Extra = Extra;

    fn parse(text: &str) -> Result<Self::Tree, Self::ParseError> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(text, None).unwrap();
        Ok(tree)
    }
}

#[derive(Debug, Clone)]
pub struct Extra {
    return_type_span: Range<usize>,
}

impl Tree<C> for treesitter_types_c::tree_sitter::Tree {
    type ParseError = treesitter_types_c::ParseError;

    fn subroutine_spans(&self) -> Vec<std::ops::Range<usize>> {
        todo!()
    }

    fn subroutine_at_span(
        &self,
        span: &std::ops::Range<usize>,
    ) -> Result<super::subroutine::Subroutine<C>, Self::ParseError> {
        todo!()
    }
}

impl Function for C {
    fn return_type_span(subroutine: &super::Subroutine<Self>) -> Option<&Range<usize>> {
        todo!()
    }
}
