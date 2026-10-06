#![cfg(feature = "c")]
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::callable::{
    AnyCallable, Callable, Descriptor, Language, Location, Name, function::Function,
};

#[derive(Debug, Clone, PartialEq)]
pub struct C;

impl Language for C {
    const NAME: &'static str = "C";
    type Kind = Kind;
}

impl AsRef<Function> for Callable<C> {
    fn as_ref(&self) -> &Function {
        let Kind::Function(function) = &self.kind;
        function
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    Function(Function),
}

impl From<Callable<C>> for AnyCallable {
    fn from(value: Callable<C>) -> Self {
        AnyCallable::C(value)
    }
}

impl Callable<C> {
    pub fn test_3(&self) {
        unreachable!()
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("failed to locate the function definition")]
    MissingFunctionDefinition,
    #[error("failed to locate the function declaration")]
    MissingFunctionDeclarator,
    #[error("failed to locate the function identifier")]
    MissingFunctionIdentifier,
    #[error(transparent)]
    Other(#[from] treesitter_types_c::ParseError),
}

impl FromStr for Callable<C> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use treesitter_types_c::*;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(s, None).unwrap();
        let translation_unit = TranslationUnit::from_node(tree.root_node(), s.as_bytes())?;
        // TODO: maybe return an error if there is more than one child node?
        let Some(TranslationUnitChildren::FunctionDefinition(function)) =
            &translation_unit.children.first()
        else {
            return Err(Self::Err::MissingFunctionDefinition);
        };

        // Extract the function name
        let Declarator::FunctionDeclarator(function_declarator) = &function.declarator else {
            return Err(Self::Err::MissingFunctionDeclarator);
        };
        let FunctionDeclaratorDeclarator::Declarator(declarator) = &function_declarator.declarator
        else {
            return Err(Self::Err::MissingFunctionDeclarator);
        };
        let Declarator::Identifier(identifier) = &**declarator else {
            return Err(Self::Err::MissingFunctionIdentifier);
        };
        let name = identifier.text().to_string();

        // Extract the function definition
        let definition = s.trim().to_string();

        // Extract the function return type
        let return_type = match &function.r#type {
            TypeSpecifier::EnumSpecifier(enum_specifier) => enum_specifier
                .name
                .as_ref()
                .map(TypeIdentifier::text)
                .unwrap_or_default(),
            TypeSpecifier::MacroTypeSpecifier(macro_type_specifier) => {
                macro_type_specifier.name.text()
            }
            TypeSpecifier::PrimitiveType(primitive_type) => primitive_type.text(),
            TypeSpecifier::SizedTypeSpecifier(sized_type_specifier) => {
                if let Some(sized_type_specifier_type) = &sized_type_specifier.r#type {
                    match sized_type_specifier_type {
                        SizedTypeSpecifierType::PrimitiveType(primitive_type) => {
                            primitive_type.text()
                        }
                        SizedTypeSpecifierType::TypeIdentifier(type_identifier) => {
                            type_identifier.text()
                        }
                    }
                } else {
                    ""
                }
            }
            TypeSpecifier::StructSpecifier(struct_specifier) => struct_specifier
                .name
                .as_ref()
                .map(TypeIdentifier::text)
                .unwrap_or_default(),
            TypeSpecifier::TypeIdentifier(type_identifier) => type_identifier.text(),
            TypeSpecifier::UnionSpecifier(union_specifier) => union_specifier
                .name
                .as_ref()
                .map(TypeIdentifier::text)
                .unwrap_or_default(),
        };
        let return_type = if return_type.is_empty() {
            None
        } else {
            Some(return_type.to_string())
        };

        Ok(Self {
            descriptor: Descriptor {
                name: Some(Name::Declared(name)),
                definition: definition,
                location: None,
            },
            kind: Kind::Function(Function { return_type }),
        })
    }
}
