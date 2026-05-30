pub mod builder;
pub(crate) mod config;
pub mod dataset;
pub mod decompiler;
pub(crate) mod docker;
mod errors;
pub mod extractor;
pub mod mapper;
pub mod pipeline;
pub mod repo;

pub use errors::Error;

pub fn hello() -> String {
    "Hello from codablellm-core!".to_string()
}
