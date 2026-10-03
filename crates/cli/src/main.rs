use codablellm_core::extractor::extract_str;

fn main() {
    let b = extract_str("contents");
    serde_json::to_string_pretty(&b);
    println!("Hello, world!");
}
