#![cfg_attr(docsrs, feature(doc_cfg))]

pub use crate::extractor::{Error as ExtractorError, Options as ExtractorOptions};
pub use crate::pipeline::{
    Options, run as create_dataset, run_with_options as create_dataset_with_options,
};

pub mod dataset;
pub mod extractor;
pub mod pipeline;
pub mod source;
pub mod transformer;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
