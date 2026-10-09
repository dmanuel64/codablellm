mod c;
mod subroutine;

use std::{
    any::Any,
    fmt::{Debug, Display},
    ops::Range,
    sync::Arc,
};

use crop::{Rope, RopeSlice};
use serde::{Deserialize, Serialize};

#[cfg(feature = "c")]
pub use crate::source::c::C;
use crate::source::subroutine::Callable;
pub use crate::source::subroutine::{
    AssociatedFunction, Function, Method, Scope, StaticMethod, Subroutine,
};

pub trait Tree<L: Language> {
    type ParseError: std::error::Error;

    fn subroutine_spans(&self) -> Vec<Range<usize>>;
    fn subroutine_at_span(&self, span: &Range<usize>) -> Result<Subroutine<L>, Self::ParseError>;
}

fn tree_subroutines<L: Language>(tree: &L::Tree) -> Vec<Result<Subroutine<L>, L::ParseError>> {
    tree.subroutine_spans()
        .iter()
        .map(|span| tree.subroutine_at_span(span))
        .collect()
}

fn tree_subroutines_strict<L: Language>(
    tree: &L::Tree,
) -> Result<Vec<Subroutine<L>>, L::ParseError> {
    tree.subroutine_spans()
        .iter()
        .map(|span| tree.subroutine_at_span(span))
        .collect()
}

pub trait Language: Sized + std::fmt::Debug + Clone + 'static {
    const NAME: &str;
    const FILE_EXTENSIONS: &[&str];

    type Tree: Tree<Self, ParseError = Self::ParseError>;
    type ParseError: std::error::Error;
    type Extra: std::fmt::Debug + Clone;

    fn parse(text: &str) -> Result<Self::Tree, Self::ParseError>;
}

pub trait Source: Debug + Any {
    fn language(&self) -> &str;
    fn text(&self) -> Rope;
    fn callables(&self) -> Vec<&dyn Callable>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "CodeRepr")]
pub struct Code<L: Language> {
    text: Rope,
    #[serde(skip)]
    subroutines: Vec<Result<Subroutine<L>, L::ParseError>>,
}

#[derive(Deserialize)]
struct CodeRepr {
    text: Rope,
}

impl<L: Language> TryFrom<CodeRepr> for Code<L> {
    type Error = L::ParseError;

    fn try_from(repr: CodeRepr) -> Result<Self, Self::Error> {
        Code::new(&repr.text.to_string())
    }
}

impl<L: Language> Source for Code<L> {
    fn language(&self) -> &str {
        L::NAME
    }

    fn text(&self) -> Rope {
        self.text.clone()
    }

    fn callables(&self) -> Vec<&dyn Callable> {
        self.subroutines
            .iter()
            .filter_map(|subroutine| subroutine.as_ref().ok())
            .map(|s| s as &dyn Callable)
            .collect()
    }
}

impl<L: Language> Code<L> {
    pub fn new(text: &str) -> Result<Self, L::ParseError> {
        let tree = L::parse(text)?;
        let text = Rope::from(text);
        let subroutines = tree_subroutines(&tree);
        Ok(Self { text, subroutines })
    }

    pub fn new_strict(text: &str) -> Result<Self, L::ParseError> {
        let tree = L::parse(text)?;
        let text = Rope::from(text);
        let subroutines = tree_subroutines_strict(&tree)?
            .into_iter()
            .map(Ok)
            .collect();
        Ok(Self { text, subroutines })
    }

    pub fn has_errors(&self) -> bool {
        self.subroutines.iter().any(Result::is_err)
    }

    pub fn subroutines(&self) -> &[Result<Subroutine<L>, L::ParseError>] {
        self.subroutines.as_slice()
    }

    pub fn find_subroutine(&self, qualified_identifier: &str) -> Option<&Subroutine<L>> {
        self.subroutines
            .iter()
            .find(|subroutine| {
                subroutine.as_ref().is_ok_and(|s| {
                    s.name()
                        .as_ref()
                        .map(RopeSlice::to_string)
                        .is_some_and(|name| name == qualified_identifier)
                })
            })
            .map(|r| r.as_ref().ok())
            .flatten()
    }

    pub fn edit_with<EditFn, R>(&mut self, f: EditFn) -> Result<R, L::ParseError>
    where
        EditFn: FnOnce(&mut Rope) -> R,
    {
        let old = self.text.clone();
        let return_value = f(&mut self.text);
        let tree = L::parse(&self.text.to_string()).inspect_err(|_| self.text = old)?;
        self.subroutines = tree_subroutines(&tree);
        Ok(return_value)
    }

    pub fn edit_with_strict<EditFn, R>(&mut self, f: EditFn) -> Result<R, L::ParseError>
    where
        EditFn: FnOnce(&mut Rope) -> R,
    {
        let old = self.text.clone();
        let return_value = f(&mut self.text);
        let tree = L::parse(&self.text.to_string()).inspect_err(|_| self.text = old.clone())?;
        self.subroutines = tree_subroutines_strict(&tree)
            .inspect_err(|_| self.text = old)?
            .into_iter()
            .map(Ok)
            .collect();
        Ok(return_value)
    }

    pub fn edit_definition(
        &mut self,
        subroutine: &Subroutine<L>,
        new_definition: &str,
    ) -> Result<(), L::ParseError> {
        self.edit_with(|code| code.replace(subroutine.span().clone(), new_definition))
    }

    pub fn edit_definition_strict(
        &mut self,
        subroutine: &Subroutine<L>,
        new_definition: &str,
    ) -> Result<(), L::ParseError> {
        self.edit_with_strict(|code| code.replace(subroutine.span().clone(), new_definition))
    }
}

impl<L: Language> Display for Code<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = &self.text;
        write!(f, "{text}")
    }
}

impl<L: Language> From<Code<L>> for AnyCode {
    fn from(value: Code<L>) -> Self {
        AnyCode(Arc::new(value))
    }
}

#[derive(Debug, Clone)]
pub struct AnyCode(Arc<dyn Source>);

impl Source for AnyCode {
    fn language(&self) -> &str {
        self.0.language()
    }

    fn text(&self) -> Rope {
        self.0.text()
    }

    fn callables(&self) -> Vec<&dyn Callable> {
        self.0.callables()
    }
}

impl AnyCode {
    pub fn as_language<L: Language>(&self) -> Option<&Code<L>> {
        let any: &dyn Any = &*self.0;
        any.downcast_ref::<Code<L>>()
    }

    #[cfg(feature = "c")]
    pub fn as_c(&self) -> Option<&Code<C>> {
        self.as_language()
    }
}
