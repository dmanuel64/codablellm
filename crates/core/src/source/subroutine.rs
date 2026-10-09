use std::path::Path;
use std::{fmt::Display, ops::Range};

use crop::{Rope, RopeSlice};

use crate::source::Language;

#[derive(Debug, Clone)]
pub struct Subroutine<L: Language> {
    code: Rope,
    name_span: Option<Range<usize>>,
    definition_span: Range<usize>,
    #[allow(unused)]
    extra: L::Extra,
}

impl<L: Language> Subroutine<L> {
    pub fn new(
        code: Rope,
        name_span: Option<Range<usize>>,
        definition_span: Range<usize>,
        extra: L::Extra,
    ) -> Self {
        Self {
            code,
            name_span,
            definition_span,
            extra,
        }
    }

    pub fn name(&self) -> Option<RopeSlice<'_>> {
        self.name_span
            .as_ref()
            .map(|span| self.code.byte_slice(span.clone()))
    }

    pub fn definition(&self) -> RopeSlice<'_> {
        self.code.byte_slice(self.definition_span.clone())
    }
}

impl<L: Language> Display for Subroutine<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = &self.code;
        write!(f, "{code}")
    }
}

impl<L: Function> Subroutine<L> {
    pub fn return_type(&self) -> Option<RopeSlice<'_>> {
        let return_type_span = L::return_type_span(self);
        return_type_span.map(|span| self.code.byte_slice(span.clone()))
    }

    pub fn is_void(&self) -> bool {
        self.return_type()
            .map(|slice| slice.to_string())
            .is_none_or(|return_type| {
                return_type.eq_ignore_ascii_case("void") || return_type == "()"
            })
    }
}

pub trait Function: Language {
    fn return_type_span(subroutine: &Subroutine<Self>) -> Option<&Range<usize>>;
}

pub trait Scope: Language {
    fn scope<'a>(subroutine: &'a Subroutine<Self>) -> &'a [&'a str];

    fn scope_separator() -> &'static str {
        "::"
    }
}

impl<L: Scope> Subroutine<L> {
    pub fn scope(&self) -> String {
        L::scope(self).join(L::scope_separator())
    }
}

pub trait AssociatedFunction: Scope + Function {
    fn associated_type_span(subroutine: &Subroutine<Self>) -> Option<&Range<usize>>;
}

impl<L: AssociatedFunction> Subroutine<L> {
    pub fn associated_type(&self) -> Option<RopeSlice<'_>> {
        let associated_type_span = L::associated_type_span(self);
        associated_type_span.map(|span| self.code.byte_slice(span.clone()))
    }
}

pub trait StaticMethod: AssociatedFunction {
    fn is_static_method(subroutine: &Subroutine<Self>) -> bool;
}

impl<L: StaticMethod> Subroutine<L> {
    pub fn is_static_method(&self) -> bool {
        L::is_static_method(self)
    }
}

pub trait Method: AssociatedFunction {
    fn is_method(subroutine: &Subroutine<Self>) -> bool;
}

impl<L: Method> Subroutine<L> {
    pub fn is_method(&self) -> bool {
        L::is_method(self)
    }
}
