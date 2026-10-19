use polars::prelude::*;

use crate::source::{AnyCode, Source};

pub struct Dataset {
    df: LazyFrame,
}

impl Dataset {
    pub fn new<C>(code: C) -> Result<Self, PolarsError>
    where
        C: IntoIterator<Item = AnyCode>,
    {
        let languages = code
            .into_iter()
            .map(|c| c.language().to_string())
            .collect::<Vec<_>>();
        Ok(Self {
            df: df!("language" => languages)?.lazy(),
        })
    }
}
