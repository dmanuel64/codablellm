#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod builder;
pub(crate) mod container;
pub mod dataset;
pub mod decompiler;
pub mod extractor;
pub mod function;
pub mod language;
pub mod mapper;
pub(crate) mod parser;
pub mod pipeline;
pub mod repo;
pub(crate) mod utils;

pub use dataset::{BinaryDataset, Dataset, SourceDataset};
pub use extractor::Transform;
pub use pipeline::{BinaryMode, Error as CodablellmError, Mode, Options, run, run_with_options};
pub use repo::Repository;
pub use utils::ProgressDisplay;
