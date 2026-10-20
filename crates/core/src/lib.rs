#![cfg_attr(docsrs, feature(doc_cfg))]

use mimalloc::MiMalloc;

pub use crate::extractor::Options as ExtractorOptions;
pub use crate::pipeline::Options;
pub use crate::pipeline::run as create_dataset;

pub mod dataset;
pub mod extractor;
pub mod pipeline;
pub mod source;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
