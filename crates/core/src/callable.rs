mod c;
mod function;

use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    ops::Deref,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[cfg(feature = "c")]
pub use crate::callable::c::{C, Kind as CKind};

pub trait Language: Clone + PartialEq {
    const NAME: &'static str;
    type Kind: Debug + Clone + PartialEq + Serialize + DeserializeOwned;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Descriptor {
    pub name: Option<Name>,
    pub definition: String,
    pub location: Option<Location>,
}

fn descriptor_name(descriptor: &Descriptor) -> Cow<'_, str> {
    match (&descriptor.name, &descriptor.location) {
        (None, None) => Cow::Owned("<anonymous>".to_string()),
        (None, Some(location)) => Cow::Owned(format!("<callable at {location}>")),
        (Some(name), _) => Cow::Borrowed(name),
    }
}

impl Display for Descriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = descriptor_name(self);
        if let Some(location) = &self.location {
            write!(f, "{name} ({location})")
        } else {
            write!(f, "{name}")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Name {
    Declared(String),
    Synthetic(String),
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Name::Declared(name) | Name::Synthetic(name) => name,
        }
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &str = self.as_ref();
        write!(f, "{name}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]

pub enum Location {
    Source { path: PathBuf, span: Span },
    Offset { path: PathBuf, offset: usize },
}

impl Location {
    pub fn path(&self) -> &Path {
        match self {
            Location::Source { path, .. } | Location::Offset { path, .. } => path,
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self.path().display();
        let location = match self {
            Location::Source { span, .. } => {
                let start_line = span.start.line;
                format!("{path}:{start_line}")
            }
            Location::Offset { offset, .. } => format!("{path}@{offset}"),
        };
        write!(f, "{location}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start: Position,
    pub end: Position,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.start;
        let end = self.end;
        write!(f, "{start}-{end}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = self.line;
        let column = self.column;
        write!(f, "{line}:{column}")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Callable<L: Language> {
    pub descriptor: Descriptor,
    inner: L::Kind,
}

impl<L: Language> Callable<L> {
    pub fn new(descriptor: Descriptor, callable: L::Kind) -> Self {
        Self {
            descriptor,
            inner: callable,
        }
    }

    pub fn name(&self) -> Cow<'_, str> {
        descriptor_name(&self.descriptor)
    }

    pub fn definition(&self) -> &str {
        &self.descriptor.definition
    }

    pub fn location(&self) -> Option<&Location> {
        self.descriptor.location.as_ref()
    }

    pub fn language(&self) -> &'static str {
        L::NAME
    }

    pub fn kind(&self) -> &L::Kind {
        &self.inner
    }
}

impl<L: Language> Deref for Callable<L> {
    type Target = L::Kind;

    fn deref(&self) -> &Self::Target {
        self.kind()
    }
}

impl<L: Language> Display for Callable<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let descriptor = &self.descriptor;
        write!(f, "{descriptor}")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnyCallable {
    #[cfg(feature = "c")]
    C(Callable<C>),
}

impl AnyCallable {
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            #[cfg(feature = "c")]
            AnyCallable::C(callable) => callable.name(),
        }
    }

    pub fn definition(&self) -> &str {
        match self {
            #[cfg(feature = "c")]
            AnyCallable::C(callable) => callable.definition(),
        }
    }

    pub fn location(&self) -> Option<&Location> {
        match self {
            #[cfg(feature = "c")]
            AnyCallable::C(callable) => callable.location(),
        }
    }

    pub fn language(&self) -> &'static str {
        match self {
            #[cfg(feature = "c")]
            AnyCallable::C(callable) => callable.language(),
        }
    }

    #[cfg(feature = "c")]
    pub fn as_c(&self) -> Option<&Callable<C>> {
        if let AnyCallable::C(callable) = self {
            Some(callable)
        } else {
            None
        }
    }
}
