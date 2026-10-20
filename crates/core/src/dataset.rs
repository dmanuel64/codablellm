use std::fmt::Display;

use polars::prelude::*;

use crate::source::{AnyCode, Callable, Source};

pub struct Dataset {
    pub df: LazyFrame,
}

impl Dataset {
    pub fn new<C>(code: C) -> Result<Self, PolarsError>
    where
        C: IntoIterator<Item = AnyCode>,
    {
        let code: Vec<_> = code.into_iter().collect();
        let (languages, names, definitions): (Vec<_>, Vec<_>, Vec<_>) = code
            .iter()
            .flat_map(|c| c.callables())
            .map(|callable| {
                (
                    callable.language().to_string(),
                    callable.name().map(|n| n.to_string()),
                    callable.definition().to_string(),
                )
            })
            .collect();
        Ok(Self {
            df: df![
            "language" => languages,
            "name" => names,
            "definition" => definitions
            ]?
            .lazy(),
        })
    }
}

impl Display for Dataset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let df = self.df.clone().collect().unwrap();
        write!(f, "{df}")
    }
}
