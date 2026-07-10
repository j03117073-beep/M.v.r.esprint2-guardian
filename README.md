Copyright © 2026 OBINNA JAMES EJIOFOR
All Rights Reserved.

# M.V.R.ESPRINT1

Rust codebase for deterministic grid assurance experiments, audit-chain utilities, and SCED offer-chain verification.

This repository is positioned as an advisory-only deterministic replay and analytical platform for post-event reconstruction, engineering review, and evidentiary analysis. It is not intended to interact with AGC, market clearing, real-time dispatch, or other live control functions.

## Current Build Snapshot

Status as verified on 2026-03-26:

- `cargo test --lib` passes
- `cargo check` passes
- `cargo test --no-run` passes
- `cargo build --bin sced_chain --bin verifier --bin demo --bin formal_proof_harness --bin dashboard --bin pilot_demo --bin scenario_runner` passes
- `cargo test --test adversarial_validation` passes
- `cargo build --bins`
- `cargo test --all`
- `cargo run --bin sced_chain -- benchmark test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv` passes
- `./scripts/boot_pilot_scenario.sh` passes
- `./scripts/boot_full_scenario.sh` passes

Release benchmark snapshot on `test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv`:

- `records_total=1152`
- `intervals=288`
- `runtime_ms=75`
- `throughput_rps=15303`
- `throughput_intervals_per_sec=3826`

## What Is Working Today

### 1. SCED Offer-Chain Verification

The most complete production path in the repository is the SCED verifier in [`src/sced_offer_chain.rs`](/workspaces/M.V.R.ESPRINT1/src/sced_offer_chain.rs) with the CLI in [`src/bin/sced_chain.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/sced_chain.rs).

It supports:

- deterministic CSV parsing and canonical ordering
- duplicate primary-key detection
- chain-hash generation and replay verification
- extended physics validation using `SYSTEM_LAMBDA`, `SHADOW_PRICE`, `SHIFT_FACTOR`, and `LMP`
- residual reporting
- interval counting
- constraint-activation event detection
- ERCOT NP3-965-ER causal decomposition from folders or ZIPs
- Phase III prediction payload sampling and validation
- benchmark mode
- formal evidentiary report generation through the attestation workflow

Available commands:

```bash
cargo run --bin sced_chain -- verify <input.csv> [expected_hash]
cargo run --bin sced_chain -- verify-full <input.csv>
cargo run --bin sced_chain -- benchmark <input.csv>
cargo run --bin sced_chain -- verify-against <input.csv> <reference.csv>
cargo run --bin sced_chain -- decompose <zip_or_folder>
cargo run --bin sced_chain -- --decompose <zip_file>
cargo run --bin sced_chain -- predict --sample
cargo run --bin sced_chain -- predict --validate <prediction.json>
```

Primary vector directory:

- [`test_vectors/`](/workspaces/M.V.R.ESPRINT1/test_vectors)

Included full-day proxy vector:

- [`test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv`](/workspaces/M.V.R.ESPRINT1/test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv)

Generator for that vector:

- [`scripts/generate_proxy_dataset.py`](/workspaces/M.V.R.ESPRINT1/scripts/generate_proxy_dataset.py)

### 2. Core Library Tests

`cargo test --lib` currently passes with 41 tests, including:

- SCED verifier/vector coverage
- constraint-system logic
- compliance checks
- protocol-driver validation
- trace/harness unit tests

### 3. Scenario Demonstration Path

The repo now includes a hardened scenario path through [`src/scenario_kernel.rs`](/workspaces/M.V.R.ESPRINT1/src/scenario_kernel.rs) and [`src/bin/scenario_runner.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/scenario_runner.rs).

It supports:

- deterministic scenario preflight validation
- fail-closed audit artifact generation
- human-readable Markdown audit tickets
- machine-readable JSON attestation logs
- clean operator stdout with `--quiet`, `--verbose`, and `--json`
- pilot and full wrapper scripts
- snapshot-based integration testing through the ISE harness

Primary commands:

```bash
cargo run --bin scenario_runner -- --mode pilot
cargo run --bin scenario_runner -- --mode full --json
make pilot-scenario
make precompile-full
make full-scenario
```

Primary artifacts:

- [`scenario_audit_ticket.md`](/workspaces/M.V.R.ESPRINT1/scenario_audit_ticket.md)
- [`scenario_attestation_log.json`](/workspaces/M.V.R.ESPRINT1/scenario_attestation_log.json)
- [`performance_ledger.json`](/workspaces/M.V.R.ESPRINT1/performance_ledger.json)
- [`logs/pilot/`](/workspaces/M.V.R.ESPRINT1/logs/pilot)
- [`logs/full/`](/workspaces/M.V.R.ESPRINT1/logs/full)

### 4. Integration Simulation Environment

The repository now includes a lightweight Integration Simulation Environment through [`src/ise.rs`](/workspaces/M.V.R.ESPRINT1/src/ise.rs) and [`src/bin/ise_runner.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/ise_runner.rs).

It is intentionally snapshot-based:

- no live input mutation during a run
- no nondeterministic randomness
- no bypass of ICCP or external-model snapshot validation

Primary command:

```bash
cargo run --bin ise_runner -- --mode accelerated --factor 60
```

Stress example:

```bash
cargo run --bin ise_runner -- --mode step --inject load-spike --inject constraint-stress
```

Primary artifacts:

- [`ise_performance_report.json`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.json)
- [`ise_performance_report.md`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.md)
- [`ise_scenario_timeline_log.jsonl`](/workspaces/M.V.R.ESPRINT1/ise_scenario_timeline_log.jsonl)

### 5. Demo and Harness Binaries

These binaries currently build and run:

- `demo`
- `dashboard`
- `formal_proof_harness`
- `pilot_demo`
- `ise_runner`
- `scenario_runner`
- `verifier`
- `sced_chain`

Their roles:

- `demo`: runs canned market-state scenarios through the demo pipeline
- `dashboard`: serves the demo dashboard, scenario API, and demo audit-ticket/report endpoints
- `formal_proof_harness`: checks simple invariant-style proofs against zero-state/TLBSS primitives
- `pilot_demo`: generates a sample attestation log, verifies it, and writes a formal evidentiary Markdown report
- `ise_runner`: replays timestamped scenario inputs through a deterministic integration harness and emits performance plus timeline reports
- `scenario_runner`: executes the demonstration scenario kernel and emits standardized audit artifacts
- `verifier`: validates JSON attestation chains for `AttestationRecord` using the shared Ed25519-backed verification path
- `sced_chain`: validates and benchmarks SCED offer-chain CSVs

### 6. TLBSS™ v2.0 — Tri-Layer Bulk-System Substrate

A complete reference implementation of the official TLBSS™ v2.0 engineering specification for deterministic assurance of bulk electric power systems. This module provides the foundational deterministic execution engine for the Minimal Viable Resonance Engine (MVRE).

**Status**: ✅ Fully conformant to official specification (Parts I, II, III)

**Key Features**:
- **Deterministic execution**: Identical evidence always produces identical results
- **Five-stage pipeline**: Evidence validation → Topology verification → State reconstruction → Operational assurance → Advisory generation
- **Evidence preservation**: Immutable audit trail with complete provenance
- **Physical constraints**: Enforces electrical engineering limits (voltage, frequency, topology)
- **Failure classification**: Five classes of failures (Class I-V) with explicit handling per spec
- **Replay identity**: Deterministic computation for reproducible execution

**Implementation**:
- Module: [`src/tlbss_v2.rs`](src/tlbss_v2.rs)
- Documentation: [`TLBSS_V2_IMPLEMENTATION.md`](TLBSS_V2_IMPLEMENTATION.md)
- Test suite: 6 comprehensive tests, all passing ✓

**Foundational Principles**:
1. Physical Primacy — Physical network is authoritative
2. Evidence Before Inference — Measurements before computation
3. Deterministic Computation — Reproducible execution guaranteed
4. Explicit Topology — Connectivity always represented
5. Operational Independence — Assessment separate from control

**Running Tests**:

```bash
cargo test --lib tlbss_v2 -- --nocapture
```

**Expected output**:
```
running 6 tests
test tlbss_v2::tests::executes_deterministically_and_replays_identically ... ok
test tlbss_v2::tests::rejects_invalid_evidence_before_topology_processing ... ok
test tlbss_v2::tests::reports_indeterminate_state_when_reconstruction_is_not_feasible ... ok
test tlbss_v2::tests::preserves_evidence_package_immutably ... ok
test tlbss_v2::tests::generates_deterministic_replay_identity ... ok
test tlbss_v2::tests::classifies_failures_correctly ... ok

test result: ok. 6 passed
```

**Specification Reference**: TLBSS™ v2.0 Official Engineering Specification (Parts I-III)  
**Framework Owner**: Obinna James Ejiofor  
**Document Classification**: Technical Standard

## Repository Shape


Key library modules exported from [`src/lib.rs`](src/lib.rs):

- TLBSS and deterministic frameworks: **`tlbss_v2`** (TLBSS™ v2.0 reference implementation)
- assurance and policy: `audit_guardian`, `regulatory_policy`, `constraint_system`, `compliance`
- simulation and harnessing: `simulation`, `simulation_harness_core`, `adversarial_harness`, `demo_pipeline`
- sovereign/runtime concepts: `sovereign_kernel`, `sovereign_trace`, `sovereign_bus`, `kernel`
- interfaces and drivers: `interface_discovery`, `protocol_drivers`, `operator_interface`, `drivers`, `sp_api`
- execution/codegen: `universal_frontend`, `ir_codegen`, `ir_backends`, `crypto_pipeline`
- SCED chain path: `sced_offer_chain`, `sced_decomposition`, `phase3_prediction`

## Recommended Commands

Single-command kernel wrappers:

```bash
make pilot
make full
make pilot-scenario
make full-scenario
make precompile-full
```

Build the known-good binaries:

```bash
cargo build --bin sced_chain --bin verifier --bin demo --bin formal_proof_harness --bin dashboard --bin pilot_demo --bin scenario_runner
```

Run the library test suite:

```bash
cargo test --lib
```

Run the SCED proxy verification path:

```bash
cargo run --bin sced_chain -- verify-full test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv
cargo run --bin sced_chain -- benchmark test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv
cargo run --bin sced_chain -- predict --sample
```

Run the demo CLI:

```bash
cargo run --bin demo -- normal
```

Generate a sample audit artifact:

```bash
cargo run --bin pilot_demo
```

This writes:

- [`pilot_attestation_log.json`](/workspaces/M.V.R.ESPRINT1/pilot_attestation_log.json)
- [`pilot_audit_ticket.md`](/workspaces/M.V.R.ESPRINT1/pilot_audit_ticket.md)

Generate the demonstration-ready scenario artifacts:

```bash
cargo run --bin scenario_runner -- --mode pilot
```

This writes:

- [`scenario_attestation_log.json`](/workspaces/M.V.R.ESPRINT1/scenario_attestation_log.json)
- [`scenario_audit_ticket.md`](/workspaces/M.V.R.ESPRINT1/scenario_audit_ticket.md)

Run the formal proof harness:

```bash
cargo run --bin formal_proof_harness
```

Kernel boot wrappers:

- [`scripts/boot_pilot.sh`](/workspaces/M.V.R.ESPRINT1/scripts/boot_pilot.sh)
- [`scripts/boot_full.sh`](/workspaces/M.V.R.ESPRINT1/scripts/boot_full.sh)
- [`Makefile`](/workspaces/M.V.R.ESPRINT1/Makefile)

## Current Constraints

- `verifier` expects a JSON file containing `Vec<AttestationRecord>` and will fail on unrelated text logs.
- default attestation mode is deterministic `mock`; TPM-backed mode is feature-gated behind `tpm`
- severe demo scenarios still need tighter truthfulness alignment so adverse labels and actual audit outcomes always match
- the scenario wrappers now verify required files and release binaries rather than enforcing a clean-git-only gate

## Supporting Docs

- [`REVIEW_PACKET.md`](/workspaces/M.V.R.ESPRINT1/REVIEW_PACKET.md)
- [`ARCHITECTURE_DESIGN.md`](/workspaces/M.V.R.ESPRINT1/ARCHITECTURE_DESIGN.md)
- [`PERFORMANCE_REPORT.md`](/workspaces/M.V.R.ESPRINT1/PERFORMANCE_REPORT.md)
- [`TECHNICAL_SPECIFICATIONS.md`](/workspaces/M.V.R.ESPRINT1/TECHNICAL_SPECIFICATIONS.md)
- [`OPERATIONAL_MANUAL.md`](/workspaces/M.V.R.ESPRINT1/OPERATIONAL_MANUAL.md)
- [`PILOT_BRIEF.md`](/workspaces/M.V.R.ESPRINT1/PILOT_BRIEF.md)
- [`CODING_TECHNICAL_FRAMEWORK.md`](/workspaces/M.V.R.ESPRINT1/CODING_TECHNICAL_FRAMEWORK.md)
