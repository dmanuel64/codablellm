#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod builder;
pub(crate) mod container;
pub mod dataset;
pub mod decompiler;
pub mod extractor;
pub mod function;
pub mod language;
pub mod mapper;
pub mod pipeline;
pub mod repo;
pub mod storage;

pub use dataset::{BinaryDataset, Dataset, SourceDataset};
pub use language::Language;
pub use pipeline::Error as CodablellmError;
pub use pipeline::{BinaryMode, Mode, Options, run, run_with_options};
pub use repo::{Location as RepoLocation, Metadata as RepoMetadata};
