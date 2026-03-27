# M.V.R.ESPRINT1 Operational Manual

This manual reflects the commands and workflows that are actually usable in the current repository state.

## Intended Use Right Now

Use this repository today for:

- SCED offer-chain verification and benchmarking
- SCED decomposition and Phase III payload validation
- deterministic library-level testing
- demo scenario playback
- dashboard serving
- scenario demonstration execution
- integration simulation and replay testing
- pilot attestation log generation
- formal evidentiary Markdown report generation
- Ed25519-backed attestation-chain verification for correctly formatted JSON inputs

## Environment Requirements

- Linux or compatible Unix-like shell environment
- Rust toolchain capable of building edition 2021 crates
- enough local disk for `target/` build artifacts and CSV vectors
- no system TPM libraries are required for the default build; deterministic `mock` attestation is the default

## Known-Good Commands

### Kernel Boot Surface

```bash
make pilot
make full
make pilot-scenario
make full-scenario
make precompile-full
```

### Build

```bash
cargo build --bin sced_chain --bin verifier --bin demo --bin formal_proof_harness --bin dashboard --bin pilot_demo --bin scenario_runner
cargo build --bins
```

### Test

```bash
cargo test --lib
cargo test --test adversarial_validation
cargo test --all
```

### SCED Verification

```bash
cargo run --bin sced_chain -- verify-full test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv
cargo run --bin sced_chain -- decompose <zip_or_folder>
```

### SCED Benchmark

```bash
cargo run --bin sced_chain -- benchmark test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv
```

Observed release benchmark snapshot on 2026-03-26:

- `records_total=1152`
- `intervals=288`
- `runtime_ms=75`
- `throughput_rps=15303`
- `throughput_intervals_per_sec=3826`

### Phase III Prediction Schema

```bash
cargo run --bin sced_chain -- predict --sample
cargo run --bin sced_chain -- predict --validate <prediction.json>
```

### Demo

```bash
cargo run --bin demo -- normal
```

### Dashboard

```bash
cargo run --bin dashboard
```

### Pilot Demo

```bash
cargo run --bin pilot_demo
```

Outputs:

- [`pilot_attestation_log.json`](/workspaces/M.V.R.ESPRINT1/pilot_attestation_log.json)
- [`pilot_audit_ticket.md`](/workspaces/M.V.R.ESPRINT1/pilot_audit_ticket.md)

### Scenario Runner

```bash
cargo run --bin scenario_runner -- --mode pilot
cargo run --bin scenario_runner -- --mode pilot --quiet
cargo run --bin scenario_runner -- --mode full --json
```

Outputs:

- [`scenario_attestation_log.json`](/workspaces/M.V.R.ESPRINT1/scenario_attestation_log.json)
- [`scenario_audit_ticket.md`](/workspaces/M.V.R.ESPRINT1/scenario_audit_ticket.md)

Flags:

- `--quiet`: minimal stdout
- `--verbose`: includes stage telemetry
- `--json`: emits machine-readable summary to stdout

### ISE Runner

```bash
cargo run --bin ise_runner -- --mode accelerated --factor 60
cargo run --bin ise_runner -- --mode realtime
cargo run --bin ise_runner -- --mode step --inject load-spike --inject constraint-stress
```

Outputs:

- [`ise_performance_report.json`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.json)
- [`ise_performance_report.md`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.md)
- [`ise_scenario_timeline_log.jsonl`](/workspaces/M.V.R.ESPRINT1/ise_scenario_timeline_log.jsonl)

Determinism guardrails:

- the ISE mutates scenario inputs only once before execution by preparing a temporary scenario variant
- replay runs against fixed snapshots only; no live mid-run mutation is allowed
- replay scheduling is deterministic and uses no randomness
- ICCP and external-model consistency checks still fail closed inside the kernel path

### Kernel Wrappers

- [`scripts/boot_pilot.sh`](/workspaces/M.V.R.ESPRINT1/scripts/boot_pilot.sh)
- [`scripts/boot_full.sh`](/workspaces/M.V.R.ESPRINT1/scripts/boot_full.sh)
- [`Makefile`](/workspaces/M.V.R.ESPRINT1/Makefile)

Behavior:

- `make pilot` runs the fail-closed pilot boot sequence
- `make full` runs the fail-closed full-mode benchmark sequence
- `make precompile-full` builds the release `sced_chain` binary required by full mode
- `make pilot-scenario` verifies workspace inputs, runs the scenario, emits the audit ticket, and confirms deterministic validation
- `make full-scenario` runs the release binaries, logs under [`logs/full/`](/workspaces/M.V.R.ESPRINT1/logs/full), and appends [`performance_ledger.json`](/workspaces/M.V.R.ESPRINT1/performance_ledger.json)

### Formal Harness

```bash
cargo run --bin formal_proof_harness
```

## Workspace Status

The wider workspace has now been explicitly re-verified:

- `cargo check` passes
- `cargo test --no-run` passes
- `cargo build --bins` passes
- `cargo test --all` passes

## SCED Operations

### Input Files

Working vectors live in:

- [`test_vectors/`](/workspaces/M.V.R.ESPRINT1/test_vectors)

Recommended full-day validation vector:

- [`test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv`](/workspaces/M.V.R.ESPRINT1/test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv)

### Modes

`verify`

- rebuilds the chain
- optionally binds to an expected final hash

`verify-full`

- verifies the file
- emits events
- performs internal 3x determinism replay

`benchmark`

- runs a passing verification path
- emits throughput and runtime metrics
- on the current proxy full-day vector: `1152` records, `288` intervals, `1165 ms`, `989 records/s`

`verify-against`

- compares the input chain against a reference CSV-derived chain

`decompose`

- ingests ERCOT disclosure folders or ZIPs
- emits deterministic constraint-level congestion attribution
- writes a decomposition CSV artifact

`predict`

- emits a canonical Phase III prediction sample
- validates prediction JSON against the raw nodal identity contract

## Verifier Operations

`verifier` expects a JSON file containing a serialized `Vec<AttestationRecord>`.

Example:

```bash
cargo run --bin verifier -- <attestation_log.json>
```

It will fail if you pass:

- plain text logs
- SCED CSV files
- JSON with a different schema

Current signature path:

- shared Ed25519-backed verification for simulation evidence
- deterministic `mock` attestation is the default runtime path
- TPM-backed mode is available only when built with the `tpm` feature

## Troubleshooting

### `cargo build --bins` fails

This command is currently green in the verified tree. A new failure likely indicates a fresh regression or local environment issue.

### `cargo test --all` fails

This command is currently green in the verified tree. A new failure likely indicates a fresh regression or local environment issue.

### `verifier` says JSON parse failed

Check that the input file is a JSON array of attestation records, not a text log.

## Operational Recommendations

- Prefer the SCED path for production-style validation work in this repo.
- Keep generated vectors under `test_vectors/`.
- Rebuild the proxy dataset with [`scripts/generate_proxy_dataset.py`](/workspaces/M.V.R.ESPRINT1/scripts/generate_proxy_dataset.py) if the benchmark vector needs regeneration.
- Use the generated Markdown audit artifact when you need a printable evidence package for operators or reviewers.
- Treat the evidence stack as deterministic software-backed by default unless you explicitly enable TPM mode.
- Expect the scenario wrappers to fail closed on missing datasets, malformed inputs, missing release binaries, or invalid attestation output.
- Treat the ISE as a deterministic integration harness, not a live simulator; if you need new stresses, add them as pre-run injections rather than runtime mutation.
