#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod builder;
pub mod dataset;
pub mod decompiler;
pub mod extractor;
pub mod function;
pub mod language;
pub mod mapper;
pub mod pipeline;
pub mod repo;
pub mod storage;

pub use language::Language;
pub use pipeline::Error as CodablellmError;
pub use pipeline::{Mode, Options, run, run_with_options};
#[cfg(any(
    feature = "github",
    feature = "gitlab",
    feature = "forgejo",
    feature = "custom-forge"
))]
pub use repo::Forge;
pub use repo::{GitRef, Metadata as RepoMetadata, Source as RepoSource};
pub use storage::FileSource;
