use polars::prelude::*;

use crate::source::{AnyCode, Source};

pub struct Entry {
    pub language: String,
    pub name: Option<String>,
    pub definition: String,
}

pub fn build<C>(code: C) -> PolarsResult<DataFrame>
where
    C: IntoIterator<Item = AnyCode>,
{
    let entries: Vec<_> = code.into_iter().flat_map(|c| c.entries()).collect();
    let (languages, names, definitions): (Vec<_>, Vec<_>, Vec<_>) = entries
        .iter()
        .map(|entry| {
            (
                entry.language.clone(),
                entry.name.clone(),
                entry.definition.clone(),
            )
        })
        .collect();
    df![
        "language" => languages,
        "name" => names,
        "definition" => definitions
    ]
}
