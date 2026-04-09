# Timing and Jitter Evidence

Date: April 9, 2026  
Host: Windows workspace (PowerShell)  
Tool: `src/bin/timing_probe.rs`

## Target

- 1 kHz loop (`target_us = 1000`)

## Baseline Run (moderate load)

Command:
- `cargo run --bin timing_probe -- --cycles 3000 --busy-iters 5000 --target-us 1000`

Results:
- `min_us=33`
- `max_us=5100`
- `mean_us=96.083`
- `p95_us=180`
- `p99_us=621`
- `overruns=15` (0.5%)

## Loaded Run (heavy load)

Command:
- `cargo run --bin timing_probe -- --cycles 3000 --busy-iters 50000 --target-us 1000`

Results:
- `min_us=436`
- `max_us=65683`
- `mean_us=1236.501`
- `p95_us=3153`
- `p99_us=9035`
- `overruns=1101` (36.7%)

## Interpretation

- Baseline run demonstrates sub‑millisecond steady loop with rare overruns on this host.
- Heavy load run shows expected degradation under stress; envelope captured for audit.
- These figures are host‑specific and should be re‑captured on the submission environment.

