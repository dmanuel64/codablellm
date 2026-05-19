pub mod builder;
pub(crate) mod config;
pub mod dataset;
pub mod decompiler;
pub(crate) mod docker;
pub mod extractor;
pub mod mapper;
pub mod pipeline;
pub mod repo;
pub mod types;

pub fn hello() -> String {
    "Hello from codablellm-core!".to_string()
}
