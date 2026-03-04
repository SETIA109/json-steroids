//! Profile string serialization with escape sequences
//! Usage: samply record target/release/examples/prof_serialize_string_escapes

use json_steroids::to_string;
use std::hint::black_box;

const ITERS: usize = 100_000;

fn main() {
    let s = "hello\nworld\t\"escaped\"\r\nand\\more\x00control\x1Fchars".to_string();
    let mut result = String::new();

    eprintln!(
        "Running {} iterations of serialize_string_with_escapes...",
        ITERS
    );
    let start = std::time::Instant::now();

    for _ in 0..ITERS {
        result = to_string(black_box(&s));
    }

    let elapsed = start.elapsed();
    eprintln!("Completed in {:?}", elapsed);
    eprintln!("Result contains \\n: {}", result.contains("\\n"));
    eprintln!("Avg: {:?} per iteration", elapsed / ITERS as u32);
}
