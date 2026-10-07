#![cfg(feature = "c")]
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use treesitter_types_c::*;

use crate::callable::{
    AnyCallable, Callable, Descriptor, Language, Name, Source, function::Function,
};

#[derive(Debug, Clone, PartialEq)]
pub struct C;

impl Language for C {
    const NAME: &'static str = "C";
    const FILE_EXTENSIONS: &[&str] = &[".c", ".h"];
    type Kind = Kind;
    type ParseError = ParseError;

    fn parse(source: &super::Source) -> Vec<Result<Callable<Self>, Self::ParseError>> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(source.text, None).unwrap();
        let tu = TranslationUnit::from_node(tree.root_node(), source.text.as_bytes()).unwrap();
        tu.children
            .iter()
            .flat_map(|node| match node {
                TranslationUnitChildren::FunctionDefinition(function_definition) => {
                    vec![function_definition]
                }
                // TranslationUnitChildren::PreprocIf(preproc_if) => todo!(),
                // TranslationUnitChildren::PreprocIfdef(preproc_ifdef) => todo!(),
                _ => vec![],
            })
            .map(|function_definition| {
                Callable::<C>::callable_from_node(function_definition, source)
            })
            .collect()
    }
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
    fn callable_from_node(
        function: &FunctionDefinition,
        source: &Source,
    ) -> Result<Callable<C>, ParseError> {
        // Extract the function name
        let Declarator::FunctionDeclarator(function_declarator) = &function.declarator else {
            return Err(<C as Language>::ParseError::MissingFunctionDeclarator);
        };
        let FunctionDeclaratorDeclarator::Declarator(declarator) = &function_declarator.declarator
        else {
            return Err(<C as Language>::ParseError::MissingFunctionDeclarator);
        };
        let Declarator::Identifier(identifier) = &**declarator else {
            return Err(<C as Language>::ParseError::MissingFunctionIdentifier);
        };
        let name = identifier.text().to_string();

        // Extract the function definition
        let definition = source.text.trim().to_string();

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

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("failed to locate the function definition")]
    MissingFunctionDefinition,
    #[error("failed to locate the function declaration")]
    MissingFunctionDeclarator,
    #[error("failed to locate the function identifier")]
    MissingFunctionIdentifier,
    #[error("expected only one definition")]
    MultipleDefinitions,
    #[error(transparent)]
    Other(#[from] treesitter_types_c::ParseError),
}

impl FromStr for Callable<C> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut callables = C::parse(&Source {
            text: s,
            path: None,
        });
        match callables.len() {
            1 => Ok(callables.remove(0)?),
            0 => Err(ParseError::MissingFunctionDefinition),
            _ => Err(ParseError::MultipleDefinitions),
        }
    }
}
