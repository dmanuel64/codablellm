use codablellm_core::callable::{AnyCallable, C, Callable};

fn test_1() -> Callable<C> {
    unreachable!()
}

fn test_2() -> AnyCallable {
    unreachable!()
}

fn main() {
    serde_json::to_string_pretty(&test_1().test_3()).unwrap();
    println!("Hello, world!");
}
