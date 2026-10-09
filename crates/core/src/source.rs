mod c;
pub mod subroutine;

use std::{any::Any, fmt::Display, ops::Range, sync::Arc};

use crop::Rope;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[cfg(feature = "c")]
pub use crate::source::c::{C, Extra as CCallable};
pub use crate::source::subroutine::Subroutine;

pub trait Tree<L: Language> {
    type ParseError: std::error::Error;

    fn subroutine_spans(&self) -> Vec<Range<usize>>;
    fn subroutine_at_span(&self, span: &Range<usize>) -> Result<Subroutine<L>, Self::ParseError>;
}

pub trait Language: Sized + std::fmt::Debug + Clone + 'static {
    const NAME: &str;
    const FILE_EXTENSIONS: &[&str];

    type Tree: Tree<Self, ParseError = Self::ParseError>;
    type ParseError: std::error::Error;
    type Extra: std::fmt::Debug + Clone;

    fn parse(text: &str) -> Result<Self::Tree, Self::ParseError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "CodeRepr", bound(deserialize = ""))]
pub struct Code<L: Language> {
    text: Rope,
    #[serde(skip)]
    subroutines: Vec<Result<Subroutine<L>, L::ParseError>>,
    #[serde(skip)]
    tree: L::Tree,
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

impl<L: Language> Code<L> {
    pub fn new(text: &str) -> Result<Self, L::ParseError> {
        let tree = L::parse(text)?;
        let text = Rope::from(text);
        let subroutines = tree
            .subroutine_spans()
            .iter()
            .map(|span| tree.subroutine_at_span(span))
            .collect();
        Ok(Self {
            text,
            subroutines,
            tree,
        })
    }

    pub fn has_errors(&self) -> bool {
        self.subroutines.iter().any(Result::is_err)
    }

    pub fn text(&self) -> Rope {
        self.text.clone()
    }

    pub fn subroutines(&self) -> &[Result<Subroutine<L>, L::ParseError>] {
        self.subroutines.as_slice()
    }
}

impl<L: Language> From<Code<L>> for AnyCode {
    fn from(value: Code<L>) -> Self {
        AnyCode(Box::new(value))
    }
}

impl<L: Language> Display for Code<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = &self.text;
        write!(f, "{text}")
    }
}

#[derive(Debug, Clone)]
pub struct AnyCode(Arc<Box<dyn Any>>);

impl AnyCode {
    pub fn as_language<L: Language>(&self) -> Option<&Code<L>> {
        self.0.downcast_ref::<Code<L>>()
    }

    #[cfg(feature = "c")]
    pub fn as_c(&self) -> Option<&Code<C>> {
        self.as_language::<C>()
    }
}
