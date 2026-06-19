pub mod builder;
pub(crate) mod config;
pub mod dataset;
pub mod decompiler;
pub mod extractor;
pub mod function;
pub mod language;
pub mod mapper;
pub mod pipeline;
pub mod repo;

pub fn hello() -> String {
    "Hello from codablellm-core!".to_string()
}
