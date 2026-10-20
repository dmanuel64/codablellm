#![cfg_attr(docsrs, feature(doc_cfg))]

use mimalloc::MiMalloc;

pub mod dataset;
pub mod extractor;
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
