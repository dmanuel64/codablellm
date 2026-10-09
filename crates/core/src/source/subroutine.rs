use std::{
    any::Any,
    fmt::{Debug, Display},
    ops::Range,
    sync::Arc,
};

use crop::RopeSlice;

use crate::source::{Language, ParsedCode};

pub trait Callable: Debug + Any {
    fn name(&self) -> Option<RopeSlice<'_>>;
    fn definition(&self) -> RopeSlice<'_>;
    fn span(&self) -> Range<usize>;
}

#[derive(Debug, Clone)]
pub struct Subroutine<L: Language> {
    source: Arc<ParsedCode<L>>,
    name_span: Option<Range<usize>>,
    definition_span: Range<usize>,
    #[allow(unused)]
    extra: L::Extra,
}
impl<L: Language> Callable for Subroutine<L> {
    fn name(&self) -> Option<RopeSlice<'_>> {
        self.name_span
            .as_ref()
            .map(|span| self.source.text.byte_slice(span.clone()))
    }

    fn definition(&self) -> RopeSlice<'_> {
        self.source.text.byte_slice(self.definition_span.clone())
    }

    fn span(&self) -> Range<usize> {
        self.definition_span.clone()
    }
}

impl<L: Language> Subroutine<L> {
    pub fn new(
        source: Arc<ParsedCode<L>>,
        name_span: Option<Range<usize>>,
        definition_span: Range<usize>,
        extra: L::Extra,
    ) -> Self {
        Self {
            source,
            name_span,
            definition_span,
            extra,
        }
    }

    pub fn source(&self) -> &Arc<ParsedCode<L>> {
        &self.source
    }

    pub fn extra(&self) -> &L::Extra {
        &self.extra
    }
}

impl<L: Language> Display for Subroutine<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let definition = self.definition();
        write!(f, "{definition}")
    }
}

pub trait Function: Language {
    fn return_type_span(subroutine: &Subroutine<Self>) -> Option<Range<usize>>;

    fn is_void(subroutine: &Subroutine<Self>) -> bool {
        subroutine
            .return_type()
            .is_none_or(|t| t == "void" || t == "()")
    }
}

impl<L: Function> Subroutine<L> {
    pub fn return_type(&self) -> Option<RopeSlice<'_>> {
        let return_type_span = L::return_type_span(self);
        return_type_span.map(|span| self.source.text.byte_slice(span.clone()))
    }

    pub fn is_void(&self) -> bool {
        L::is_void(self)
    }
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
    fn associated_type_span(subroutine: &Subroutine<Self>) -> Option<Range<usize>>;
}

impl<L: AssociatedFunction> Subroutine<L> {
    pub fn associated_type(&self) -> Option<RopeSlice<'_>> {
        let associated_type_span = L::associated_type_span(self);
        associated_type_span.map(|span| self.source.text.byte_slice(span.clone()))
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
