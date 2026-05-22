# M.V.R.ESPRINT1 Manual

Version: 0.1.0
Last updated: May 22, 2026

## Purpose

This manual provides the operator, engineer, and verifier reference for M.V.R.ESPRINT1. It describes the repository architecture, runtime behavior, deterministic validation workflows, ERCOT telemetry normalization, and the replay equivalence proof path.

## Scope

M.V.R.ESPRINT1 is a pilot-grade deterministic assurance framework for energy-grid evaluation workflows. It supports:

- canonical telemetry ingestion and normalization
- attested operational snapshots
- deterministic semantic evaluation
- replay equivalence proof cycles
- SCED-style verification and audit trace generation
- local dashboard monitoring and pilot scenario execution

## System Overview

M.V.R.ESPRINT1 is implemented in Rust and structured as a workspace centered on the `m_v_r_esprint1` crate. The core runtime enforces deterministic behavior through canonical hashing, semantic provenance, and trace attestation.

Key capabilities:

- deterministic telemetry normalization from ERCOT datasets
- semantic and topology identity propagation
- sovereign trace attestation for every execution cycle
- replay harness to prove equivalence across repeated executions
- validation of admissibility, protocol violations, and safety boundaries

## Repository Structure

Important top-level files and folders:

- `Cargo.toml` / `rust-toolchain.toml`: Rust build configuration
- `README.md`: project overview and quick start
- `OPERATIONAL_MANUAL.md`: operator runbook
- `TECHNICAL_SPECIFICATIONS.md`: architecture and interface details
- `src/`: core runtime source code
- `src/bin/`: executable entrypoints
- `docs/`: supporting documentation and verification reports
- `artifacts/`: sample inputs, outputs, audit evidence
- `Grid and Market Conditions/`: ERCOT dataset sources

## Execution Environment

Verified environment:

- Ubuntu 24.04 (WSL2 preferred)
- `rustc 1.94.1`
- `cargo 1.94.1`
- `build-essential`, `pkg-config`, `libssl-dev`

### Native dependency installation

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev
```

### Workspace verification

```bash
cargo check --message-format short
```

## Build and Install

Clone the repository and build the default release:

```bash
git clone https://github.com/mvre-sprint1/M.v.r.esprint1-g.git
cd M.v.r.esprint1-g
cargo build --release
```

Optional TPM-enabled build:

```bash
cargo build --release --features tpm
```

## Main Binaries

The following entrypoints are implemented in `src/bin/`:

- `pilot_demo`
- `verifier`
- `sced_chain`
- `demo`
- `dashboard`
- `formal_proof_harness`
- `trace_replay`

## Operational Workflows

### 1. Deterministic SCED Chain Verification

This workflow verifies canonical SCED CSV inputs and the resulting hash chain.

```bash
cargo run --bin sced_chain -- verify artifacts/sample_sced.csv
```

Expected behavior:

- parse input CSV
- compute deterministic canonical hash
- verify expected chain identity when provided

### 2. Attestation and Trace Generation

The `pilot_demo` binary generates operational attestation records and writes them to a log file.

```bash
cargo run --bin pilot_demo
cargo run --bin verifier pilot_attestation_log.json
```

The verifier checks:

- trace continuity
- hash-chain integrity
- semantic and governance metadata

### 3. Demo Scenario Execution

Run deterministic synthetic scenarios through the core runtime.

```bash
cargo run --bin demo -- normal
cargo run --bin demo -- all
```

Supported scenarios include:

- `normal`
- `reserve`
- `capacity`
- `network`
- `collapse`
- `all`

### 4. Dashboard Monitoring

Start the dashboard service for local monitoring.

```bash
cargo run --bin dashboard
```

Endpoints:

- `http://127.0.0.1:3000/`
- `http://127.0.0.1:3000/health`

### 5. ERCOT Telemetry Normalization and Replay Equivalence

A new replay path ingests ERCOT data, normalizes it, and evaluates deterministic equivalence.

The replay harness is executed with:

```bash
cargo run --bin trace_replay -- ercot "Grid and Market Conditions/system-wide-demand.csv"
```

This path performs:

- parsing ERCOT CSV telemetry
- canonical ordering by timestamp
- provenance hash and stream identity generation
- deterministic operational snapshot creation
- sovereign trace attestation
- repeated replay iterations
- equivalence verification of snapshot and trace identities

## Canonical Telemetry Normalization

The normalization pipeline ensures:

- fixed ordering of telemetry points by UTC timestamp
- deterministic serialization of every point
- provenance hashing using source metadata and timeline bounds
- canonical stream identity derived from normalized point sequence

Normalized telemetry is then consumed by the operational snapshot builder for semantic validation and trace attestation.

## Attestation and Trace Semantics

Every operational cycle produces a `SovereignTrace` containing:

- requested and actual setpoints
- governance mode and legal citation
- topology identity and lineage
- semantic specification identity and version
- snapshot identity
- evaluation verdict
- replay equivalence metadata
- hash over the complete trace record

This trace is the core auditable artifact for deterministic compliance.

## Development Notes

### Source boundaries

The codebase enforces isolation of canonical hashing logic. Only core boundary files may reference `canonical_core` directly. External modules use local SHA256 wrappers to preserve the boundary.

### Reproducibility

The system is designed so that repeated replay of the same dataset yields identical:

- snapshot identity
- trace hash
- semantic configuration identity

A failed equivalence proof indicates non-determinism or state mutation outside the canonical pipeline.

## Troubleshooting

### Build issues

- verify Rust and Cargo versions
- confirm native dependencies are installed
- run `cargo check --message-format short`

### Attestation or verifier failures

- ensure input JSON is complete and valid
- verify hash chain continuity
- regenerate records using `pilot_demo`

### SCED chain failures

- validate CSV schema and field presence
- check numeric parsing and source formatting
- remove duplicate or malformed rows

### Dashboard unreachable

- ensure process is running
- confirm port `3000` is available
- restart the dashboard binary

## Validation Checklist

1. `cargo check --message-format short`
2. `cargo run --bin sced_chain -- verify artifacts/sample_sced.csv`
3. `cargo run --bin pilot_demo`
4. `cargo run --bin verifier pilot_attestation_log.json`
5. `cargo run --bin trace_replay -- ercot "Grid and Market Conditions/system-wide-demand.csv"`

## Related Documentation

- `README.md`
- `OPERATIONAL_MANUAL.md`
- `TECHNICAL_SPECIFICATIONS.md`
- `PERFORMANCE_REPORT.md`
- `docs/UBUNTU_WSL_SETUP.md`
- `docs/BUILD_ALIGNMENT.md`
