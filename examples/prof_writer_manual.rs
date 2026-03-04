//! Profile JsonWriter API for manual object construction
//! Usage: samply record target/release/examples/prof_writer_manual

use json_steroids::JsonWriter;
use std::hint::black_box;

const ITERS: usize = 100_000;

fn main() {
    let mut result = String::new();

    eprintln!("Running {} iterations of writer_manual_object...", ITERS);
    let start = std::time::Instant::now();

    for _ in 0..ITERS {
        let mut w = JsonWriter::new();
        w.begin_object();
        w.write_key("id");
        w.write_i64(black_box(42));
        w.write_comma();
        w.write_key("name");
        w.write_string(black_box("Alice"));
        w.write_comma();
        w.write_key("score");
        w.write_f64(black_box(99.5));
        w.write_comma();
        w.write_key("active");
        w.write_bool(black_box(true));
        w.end_object();
        result = w.into_string();
    }

    let elapsed = start.elapsed();
    eprintln!("Completed in {:?}", elapsed);
    eprintln!("Result contains Alice: {}", result.contains("Alice"));
    eprintln!("Avg: {:?} per iteration", elapsed / ITERS as u32);
}
