# M.V.R.ESPRINT1

Deterministic assurance overlay for energy-grid evaluation workflows.

Build contract version: `0.1.0` (from `Cargo.toml`)
Last updated: April 7, 2026

## What this project provides

M.V.R.ESPRINT1 is a Rust workspace focused on deterministic evidence generation for pilot and evaluation scenarios. The current implementation emphasizes:

- Attestation record generation and verification
- Deterministic SCED-style CSV normalization and hash-chain validation
- Reproducible demo scenarios
- Local dashboard and health endpoints for pilot walkthroughs

## Repository documents

- `README.md`: high-level overview (this file)
- `OPERATIONAL_MANUAL.md`: operator runbook
- `TECHNICAL_SPECIFICATIONS.md`: architecture and interface details
- `PERFORMANCE_REPORT.md`: current performance snapshot and metrics
- `docs/BUILD_ALIGNMENT.md`: what is verified against the current build
- `docs/UBUNTU_WSL_SETUP.md`: verified Ubuntu 24.04 WSL setup

## Verified environment

- Ubuntu 24.04 (WSL2)
- `rustc 1.94.1`
- `cargo 1.94.1`
- Native packages: `build-essential`, `pkg-config`, `libssl-dev`

## Quick start

```bash
git clone https://github.com/obienova/M.V.R.ESPRINT1.git
cd M.V.R.ESPRINT1
cargo check --message-format short
```

## Main binaries

From `src/bin`:

- `pilot_demo`
- `verifier`
- `sced_chain`
- `demo`
- `dashboard`
- `formal_proof_harness`

## Common commands

```bash
cargo run --bin sced_chain -- verify artifacts/sample_sced.csv
cargo run --bin pilot_demo
cargo run --bin verifier pilot_attestation_log.json
cargo run --bin dashboard
```

## Build-verified behavior snapshot (April 7, 2026)

- `sced_chain`: requires `verify <input.csv> [expected_hash]`
- `pilot_demo`: generates `pilot_attestation_log.json` and runs verifier
- `demo`: scenarios include `normal`, `reserve`, `capacity`, `network`, `collapse`, `all`
- `dashboard`: long-running service on `127.0.0.1:3000` with `/health`
- `dashboard`, `demo`, and `sced_chain` do not provide a dedicated `--help` interface

## Build features

- default: deterministic local evaluation flows
- `tpm`: enables optional TPM-backed signer integration

```bash
cargo build --release --features tpm
```

## Safety posture

- `#![deny(unsafe_code)]` in the core project
- deterministic verification workflows
- hash-linked attestation and trace artifacts

## Notes

- This repository currently documents pilot/evaluation behavior, not production autonomous control deployment.
- For operating procedures and troubleshooting, use `OPERATIONAL_MANUAL.md`.
