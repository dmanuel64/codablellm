mod c;
mod subroutine;

use std::{
    any::Any,
    fmt::{Debug, Display},
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

pub trait Source: Debug + Any {
    fn language(&self) -> &str;
    fn text(&self) -> &Rope;
    fn callables(&self) -> Vec<&dyn Callable>;
}

impl Display for dyn Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = self.text();
        write!(f, "{text}")
    }
}

pub trait Language: Sized + Debug + Clone + 'static {
    const NAME: &str;
    const FILE_EXTENSIONS: &[&str];

    type Tree: Debug;
    type ParseError: std::error::Error;
    type Extra: std::fmt::Debug + Clone;

    fn reparse(text: &str, old_tree: Option<&Self::Tree>) -> Result<Self::Tree, Self::ParseError>;
    fn subroutines(code: &Arc<ParsedCode<Self>>)
    -> Vec<Result<Subroutine<Self>, Self::ParseError>>;

    fn parse(text: &str) -> Result<Self::Tree, Self::ParseError> {
        Self::reparse(text, None)
    }
}

// TODO: ParsedCode name is somewhat confusing with higher-level Code<L>
#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedCode<L: Language> {
    text: Rope,
    #[serde(skip)]
    tree: L::Tree,
}

impl<L: Language> ParsedCode<L> {
    pub fn tree(&self) -> &L::Tree {
        &self.tree
    }
}

type Subroutines<L> = Arc<[Result<Subroutine<L>, <L as Language>::ParseError>]>;

#[derive(Debug, Clone)]
pub struct Code<L: Language> {
    parsed: Arc<ParsedCode<L>>,
    subroutines: Subroutines<L>,
}

impl<L: Language> Serialize for Code<L> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.parsed.text.serialize(s)
    }
}

impl<'de, L: Language> Deserialize<'de> for Code<L> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = Rope::deserialize(d)?;
        Self::build(text, false, None).map_err(serde::de::Error::custom)
    }
}

impl<L: Language> Source for Code<L> {
    fn language(&self) -> &str {
        L::NAME
    }

    fn text(&self) -> &Rope {
        &self.parsed.text
    }

    fn callables(&self) -> Vec<&dyn Callable> {
        self.subroutines
            .iter()
            .filter_map(|s| s.as_ref().ok())
            .map(|s| s as &dyn Callable)
            .collect()
    }
}

impl<L: Language> Code<L> {
    fn build(text: Rope, strict: bool, old_tree: Option<&L::Tree>) -> Result<Self, L::ParseError> {
        let tree = L::reparse(&text.to_string(), old_tree)?;
        let parsed = Arc::new(ParsedCode { text, tree });
        let mut subroutines = L::subroutines(&parsed);
        if strict {
            subroutines = subroutines
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(Ok)
                .collect();
        }
        Ok(Self {
            parsed,
            subroutines: subroutines.into(),
        })
    }

    pub fn new(text: &str) -> Result<Self, L::ParseError> {
        Self::build(Rope::from(text), false, None)
    }

    pub fn new_strict(text: &str) -> Result<Self, L::ParseError> {
        Self::build(Rope::from(text), true, None)
    }

    pub fn has_errors(&self) -> bool {
        self.subroutines.iter().any(Result::is_err)
    }

    pub fn subroutines(&self) -> &[Result<Subroutine<L>, L::ParseError>] {
        &self.subroutines
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

    fn edit<R>(
        &mut self,
        f: impl FnOnce(&mut Rope) -> R,
        strict: bool,
    ) -> Result<R, L::ParseError> {
        let mut text = self.parsed.text.clone();
        let r = f(&mut text);
        *self = Self::build(text, strict, Some(&self.parsed.tree))?;
        Ok(r)
    }

    pub fn edit_with<R>(&mut self, f: impl FnOnce(&mut Rope) -> R) -> Result<R, L::ParseError> {
        self.edit(f, false)
    }

    pub fn edit_with_strict<R>(
        &mut self,
        f: impl FnOnce(&mut Rope) -> R,
    ) -> Result<R, L::ParseError> {
        self.edit(f, true)
    }

    pub fn edit_definition(&mut self, s: &Subroutine<L>, new: &str) -> Result<(), L::ParseError> {
        debug_assert!(
            Arc::ptr_eq(s.source(), &self.parsed),
            "subroutine is from an older version"
        );
        self.edit_with(|text| text.replace(s.span().clone(), new))
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
        let source: &dyn Source = self;
        write!(f, "{source}")
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

    fn text(&self) -> &Rope {
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

impl Display for AnyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source = &self.0;
        write!(f, "{source}")
    }
}
