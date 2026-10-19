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
        let languages: Vec<_> = code
            .into_iter()
            .flat_map(|c| {
                c.callables()
                    .iter()
                    .map(|callable| callable.language().to_string())
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok(Self {
            df: df!("language" => languages)?.lazy(),
        })
    }
}

impl Display for Dataset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let df = self.df.clone().collect().unwrap();
        write!(f, "{df}")
    }
}
