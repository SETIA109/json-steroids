//! Profile serialization of large array (1000 elements)
//! Usage: samply record target/release/examples/prof_serialize_large_array

use json_steroids::to_string;
use std::hint::black_box;

const ITERS: usize = 100_000;

fn main() {
    let data: Vec<i64> = (0..1_000).map(|i| i * 7 - 3).collect();
    let mut result = String::new();

    let iterations = ITERS / 10;
    eprintln!(
        "Running {} iterations of serialize_large_array...",
        iterations
    );
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        result = to_string(black_box(&data));
    }

    let elapsed = start.elapsed();
    eprintln!("Completed in {:?}", elapsed);
    eprintln!("JSON length: {}", result.len());
    eprintln!("Avg: {:?} per iteration", elapsed / iterations as u32);
}
