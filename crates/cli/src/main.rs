use codablellm_core::callable::{AnyCallable, C, Callable};
use std::str::FromStr;

fn test_1() -> Callable<C> {
    unreachable!()
}

fn test_2() -> AnyCallable {
    unreachable!()
}

fn main() {
    let func: Callable<C> = Callable::from_str("int main() { return 0; }").unwrap();
    let out = serde_json::to_string_pretty(&func).unwrap();
    println!("{func}");
    println!("{out}");
}
