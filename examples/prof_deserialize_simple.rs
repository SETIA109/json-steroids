//! Profile deserialization of simple struct
//! Usage: samply record target/release/examples/prof_deserialize_simple

use json_steroids::{
    from_str, to_string, writer, Json, JsonDeserialize, JsonError, JsonParser, JsonSerialize,
    JsonWriter, Result,
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
    let json = to_string(&data);
    let mut result: Option<SimpleStruct> = None;

    eprintln!("Running {} iterations of deserialize_simple...", ITERS);
    eprintln!("Input: {}", json);
    let start = std::time::Instant::now();

    for _ in 0..ITERS {
        result = Some(from_str::<SimpleStruct>(black_box(&json)).unwrap());
    }

    let elapsed = start.elapsed();
    eprintln!("Completed in {:?}", elapsed);
    eprintln!("Result: {:?}", result.unwrap());
    eprintln!("Avg: {:?} per iteration", elapsed / ITERS as u32);
}
