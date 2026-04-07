# M.V.R.ESPRINT1 Technical Specifications

Version: 0.2.0  
Last updated: April 7, 2026

## 1. System scope

M.V.R.ESPRINT1 is a Rust-based deterministic assurance overlay for energy-grid evaluation workflows. The implemented focus is deterministic evidence generation, chain verification, and reproducible pilot execution.

## 2. Technical baseline

- Language: Rust 2021 edition
- Crate version: `0.1.0`
- Safety posture: `#![deny(unsafe_code)]`
- Verified runtime environment: Ubuntu 24.04 (WSL2), `rustc 1.94.1`, `cargo 1.94.1`

## 3. Architecture

High-level flow:

```text
Input artifacts / scenarios
  -> Deterministic normalization and evaluation
  -> Hash-chain and attestation creation
  -> Verifier replay and integrity checks
  -> Operator review via CLI or dashboard
```

Core module families in `src/` include:

- deterministic kernel and runtime layers
- constraint and admissibility logic
- audit and trace components
- regulatory and compliance mapping utilities
- CSV/SCED deterministic verification pipeline
- telemetry and dashboard support modules

## 4. Binary interfaces

Implemented binaries in `src/bin`:

- `pilot_demo`: generates sample attestation records and verifier handoff workflow
- `verifier`: validates attestation chain and integrity
- `sced_chain`: deterministic SCED CSV normalization and chain verification
- `demo`: scenario execution commands
- `dashboard`: local Axum/Tokio dashboard service
- `formal_proof_harness`: proof-oriented harness entry point

## 5. Configuration interface

Environment variable:

- `SIGNER_MODE`
  - `simulation`
  - `tpm` (requires `tpm` feature)

Cargo feature flags:

- `default = []`
- `tpm = ["dep:tss-esapi"]`

## 6. Data and verification formats

## Attestation chain model

Logical fields used by attestation records include:

- `decision_hash`
- `pcr_digest`
- `signature`
- `timestamp`
- `prev_hash`

Verification requirements:

- signature verification
- hash-link continuity
- monotonic timestamp progression

## SCED chain input model

- Input format: CSV
- reference input: `artifacts/sample_sced.csv`
- deterministic operations:
  - schema validation
  - type parsing and normalization
  - deterministic ordering
  - chain reconstruction
  - expected-hash comparison (optional)

## 7. Dependencies

From `Cargo.toml`:

- `anyhow`
- `csv`
- `serde` + `derive`
- `serde_json`
- `sha2`
- `hex`
- `ed25519-dalek`
- `tempfile`
- `core_affinity`
- `axum`
- `tokio` with `full`
- optional: `tss-esapi`

System dependencies for verified Linux/WSL path:

- `build-essential`
- `pkg-config`
- `libssl-dev`

## 8. Performance characteristics

Current implementation guarantees and expectations:

- deterministic replay behavior is the primary requirement
- integrity correctness is prioritized over throughput
- repeat runs on identical inputs should converge to identical final hashes

The current numeric snapshot for operations-message analytics is documented in `PERFORMANCE_REPORT.md`.

## 9. Security posture

- SHA-256-based chain hashing
- signature-backed attestation model
- optional TPM signer path via Cargo feature
- tamper-evident linkage through `prev_hash` chaining

## 10. Compliance alignment (implementation intent)

The codebase and docs map to pilot-oriented assurance objectives aligned with NERC/CIP-style evidence and replayability expectations. This document describes implemented technical behavior, not certification status.

## 11. Validation commands

```bash
cargo check --message-format short
cargo run --bin sced_chain -- verify artifacts/sample_sced.csv
cargo run --bin pilot_demo
cargo run --bin verifier pilot_attestation_log.json
cargo run --bin dashboard
```

## 12. Related references

- `README.md`
- `OPERATIONAL_MANUAL.md`
- `PERFORMANCE_REPORT.md`
- `docs/UBUNTU_WSL_SETUP.md`
- `data/ercot_candy_report.md`
