//! Profile serialization of simple struct
//! Usage: samply record target/release/examples/prof_serialize_simple

use json_steroids::{
    to_string, writer, Json, JsonDeserialize, JsonError, JsonParser, JsonSerialize, JsonWriter,
    Result,
};
use std::hint::black_box;

const ITERS: usize = 100_000;

#[derive(Debug, Json)]
struct SimpleStruct {
    name: String,
    value: i64,
    active: bool,
}

fn main() {
    let data = SimpleStruct {
        name: "json-steroids".to_string(),
        value: 42,
        active: true,
    };
    let mut result = String::new();

    eprintln!("Running {} iterations of serialize_simple...", ITERS);
    let start = std::time::Instant::now();

    for _ in 0..ITERS {
        result = to_string(black_box(&data));
    }

    let elapsed = start.elapsed();
    eprintln!("Completed in {:?}", elapsed);
    eprintln!("Last output: {}", result);
    eprintln!("Avg: {:?} per iteration", elapsed / ITERS as u32);
}
