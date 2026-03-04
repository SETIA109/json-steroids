//! Profile round-trip (serialize + deserialize) of complex struct
//! Usage: samply record target/release/examples/prof_roundtrip_complex

use json_steroids::{
    from_str, to_string, writer, Json, JsonDeserialize, JsonError, JsonParser, JsonSerialize,
    JsonWriter, Result,
};
use std::hint::black_box;

const ITERS: usize = 100_000;

#[derive(Debug, Json)]
struct Metadata {
    created: String,
    updated: String,
    version: u32,
}

#[derive(Debug, Json)]
struct ComplexStruct {
    id: u64,
    name: String,
    score: f64,
    tags: Vec<String>,
    metadata: Metadata,
    values: Vec<i32>,
    maybe: Option<String>,
}

fn main() {
    let data = ComplexStruct {
        id: 9_999_999,
        name: "Profiling Subject".to_string(),
        score: 98.6,
        tags: vec![
            "rust".to_string(),
            "json".to_string(),
            "fast".to_string(),
            "zero-copy".to_string(),
        ],
        metadata: Metadata {
            created: "2026-01-01T00:00:00Z".to_string(),
            updated: "2026-02-25T12:00:00Z".to_string(),
            version: 7,
        },
        values: (0..20).collect(),
        maybe: Some("optional payload".to_string()),
    };

    eprintln!("Running {} iterations of roundtrip_complex...", ITERS / 2);
    let start = std::time::Instant::now();

    for _ in 0..(ITERS / 2) {
        let json = to_string(black_box(&data));
        let rt = from_str::<ComplexStruct>(black_box(&json)).unwrap();
        let _ = black_box(rt);
    }

    let elapsed = start.elapsed();
    eprintln!("Completed in {:?}", elapsed);
    eprintln!("Avg: {:?} per iteration", elapsed / (ITERS / 2) as u32);
}
