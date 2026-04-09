#![deny(unsafe_code)]

use std::env;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct Stats {
    count: usize,
    min_us: u128,
    max_us: u128,
    mean_us: f64,
    p95_us: u128,
    p99_us: u128,
    overruns: usize,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cycles = parse_arg_usize(&args, "--cycles").unwrap_or(5_000);
    let busy_iters = parse_arg_usize(&args, "--busy-iters").unwrap_or(5_000);
    let target_us = parse_arg_u128(&args, "--target-us").unwrap_or(1_000);

    let stats = run_probe(cycles, busy_iters, target_us);
    println!("timing_probe");
    println!("cycles={}", stats.count);
    println!("target_us={}", target_us);
    println!("min_us={}", stats.min_us);
    println!("max_us={}", stats.max_us);
    println!("mean_us={:.3}", stats.mean_us);
    println!("p95_us={}", stats.p95_us);
    println!("p99_us={}", stats.p99_us);
    println!("overruns={}", stats.overruns);
}

fn run_probe(cycles: usize, busy_iters: usize, target_us: u128) -> Stats {
    let mut samples: Vec<u128> = Vec::with_capacity(cycles);
    let mut overruns = 0usize;

    for _ in 0..cycles {
        let start = Instant::now();
        let mut acc: u64 = 0;
        for _ in 0..busy_iters {
            acc = acc.wrapping_add(1);
        }
        let _ = acc;
        let elapsed = start.elapsed().as_micros();
        if elapsed > target_us {
            overruns += 1;
        }
        samples.push(elapsed);

        if elapsed < target_us {
            let sleep_us = target_us - elapsed;
            std::thread::sleep(Duration::from_micros(sleep_us as u64));
        }
    }

    samples.sort_unstable();
    let count = samples.len();
    let min_us = *samples.first().unwrap_or(&0);
    let max_us = *samples.last().unwrap_or(&0);
    let mean_us = samples.iter().map(|v| *v as f64).sum::<f64>() / count as f64;
    let p95_us = percentile(&samples, 0.95);
    let p99_us = percentile(&samples, 0.99);

    Stats {
        count,
        min_us,
        max_us,
        mean_us,
        p95_us,
        p99_us,
        overruns,
    }
}

fn percentile(sorted: &[u128], p: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx]
}

fn parse_arg_u128(args: &[String], key: &str) -> Option<u128> {
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == key {
            return args.get(i + 1)?.parse::<u128>().ok();
        }
        i += 1;
    }
    None
}

fn parse_arg_usize(args: &[String], key: &str) -> Option<usize> {
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == key {
            return args.get(i + 1)?.parse::<usize>().ok();
        }
        i += 1;
    }
    None
}
