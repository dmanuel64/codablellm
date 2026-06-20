use polars::prelude::*;
use thiserror::Error;

use crate::function::Function;

#[derive(Debug, Error)]
#[error("failed to create dataset: {0}")]
pub struct Error(#[from] PolarsError);

pub trait Dataset {
    fn df(&self) -> &DataFrame;
    fn df_mut(&mut self) -> &mut DataFrame;
}

pub struct SourceDataset {
    df: DataFrame,
}

impl Dataset for SourceDataset {
    fn df(&self) -> &DataFrame {
        &self.df
    }

    fn df_mut(&mut self) -> &mut DataFrame {
        &mut self.df
    }
}

impl SourceDataset {
    pub fn new(functions: &Vec<Function>) -> Result<Self, Error> {
        let names: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        let definitions: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        let df = df!(
            "name" => names,
            "definitions" => definitions,
        )
        .map_err(Error::from)?;
        Ok(Self { df })
    }
}

pub struct BinaryDataset {
    df: DataFrame,
}

impl Dataset for BinaryDataset {
    fn df(&self) -> &DataFrame {
        &self.df
    }

    fn df_mut(&mut self) -> &mut DataFrame {
        &mut self.df
    }
}
