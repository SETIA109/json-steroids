//! Profile realistic API payload parsing (stress test)
//! Usage: samply record target/release/examples/prof_realistic_api

use json_steroids::parse;
use std::hint::black_box;

const ITERS: usize = 100_000;

fn main() {
    let json = r#"{
        "status": "ok",
        "code": 200,
        "data": {
            "users": [
                {"id": 1, "name": "Alice", "email": "alice@example.com", "active": true, "score": 9.8},
                {"id": 2, "name": "Bob",   "email": "bob@example.com",   "active": false,"score": 7.2},
                {"id": 3, "name": "Carol", "email": "carol@example.com", "active": true, "score": 8.5},
                {"id": 4, "name": "Dave",  "email": "dave@example.com",  "active": true, "score": 6.1},
                {"id": 5, "name": "Eve",   "email": "eve@example.com",   "active": false,"score": 9.9}
            ],
            "total": 5,
            "page": 1,
            "per_page": 20
        },
        "meta": {
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "latency_ms": 12,
            "cached": false
        }
    }"#;

    let mut count = 0usize;

    eprintln!("Running {} iterations of realistic_api_payload...", ITERS);
    let start = std::time::Instant::now();

    for _ in 0..ITERS {
        let v = parse(black_box(json)).unwrap();
        count = v["data"]["users"].as_array().unwrap().len();
    }

    let elapsed = start.elapsed();
    eprintln!("Completed in {:?}", elapsed);
    eprintln!("Users count: {}", count);
    eprintln!("Avg: {:?} per iteration", elapsed / ITERS as u32);
}
