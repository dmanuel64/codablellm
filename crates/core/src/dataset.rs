use polars::prelude::*;

use crate::source::{AnyCode, Source};

pub fn build<C>(code: C) -> PolarsResult<DataFrame>
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
    df![
        "language" => languages,
        "name" => names,
        "definition" => definitions
    ]
}
