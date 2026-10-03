#![cfg(feature = "c")]
use std::{convert::Infallible, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::subroutine::{Descriptor, Subroutine, function};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function {
    descriptor: Descriptor,
}

#[typetag::serde]
impl Subroutine for Function {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl function::Function for Function {
    fn return_type(&self) -> &str {
        todo!()
    }
}

impl FromStr for Function {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use treesitter_types_c::*;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(s, None).unwrap();
    }
}
