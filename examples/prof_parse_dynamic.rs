//! Profile parsing of dynamic JSON values
//! Usage: samply record target/release/examples/prof_parse_dynamic

use json_steroids::{parse, JsonValue};
use std::hint::black_box;

const ITERS: usize = 100_000;

fn main() {
    let json = r#"{"id":1,"name":"Alice","scores":[95,87,92],"meta":{"active":true,"ratio":0.98}}"#;
    let mut result: Option<JsonValue> = None;

    eprintln!("Running {} iterations of parse_dynamic...", ITERS);
    let start = std::time::Instant::now();

    for _ in 0..ITERS {
        result = Some(parse(black_box(json)).unwrap());
    }

    let elapsed = start.elapsed();
    eprintln!("Completed in {:?}", elapsed);
    eprintln!("Result is object: {}", result.unwrap().is_object());
    eprintln!("Avg: {:?} per iteration", elapsed / ITERS as u32);
}
