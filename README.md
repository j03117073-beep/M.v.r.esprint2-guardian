Copyright © 2026 OBINNA JAMES EJIOFOR
All Rights Reserved.

# M.V.R.ESPRINT1

Rust codebase for deterministic grid assurance experiments, audit-chain utilities, and SCED offer-chain verification.

## Phase III Audit Readiness

**Status**: ✅ Framework Activated and Ready for Independent Review

Phase III audit framework: **Start at [phase_iii/INDEX.md](phase_iii/INDEX.md)** ← External auditors begin here

**For independent reviewers**: See [phase_iii/REVIEWER_START.md](phase_iii/REVIEWER_START.md)

### Phase III Summary
- 12 operational claims documented and traceable
- 7/12 claims validated (58%)
- 10/14 evidence artifacts reproducible
- 5 invariants verified, 7 unresolved (transparent)
- Complete audit framework for external reviewers

## Current Build Snapshot

Status as verified on 2026-03-26:

- `cargo test --lib` passes
- `cargo check` passes
- `cargo test --no-run` passes
- `cargo build --bin verifier --bin demo --bin formal_proof_harness --bin dashboard --bin pilot_demo --bin audit_ticket_verifier` passes
- `cargo test --test adversarial_validation` passes
- `cargo build --bins`
- `cargo test --all`
- `cargo run --bin audit_ticket_verifier -- verify-ticket audit_ticket.json` passes
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

The most complete production path in the repository is the SCED verifier in [`src/sced_offer_chain.rs`](/workspaces/M.V.R.ESPRINT1/src/sced_offer_chain.rs).

Note: The `sced_chain` CLI (`src/bin/sced_chain.rs`) referenced in older documentation is not present in this checkout and is considered archived for the current baseline. The current baseline instead emphasizes `verifier`, `audit_ticket_verifier`, and related deterministic audit workflows. If `sced_chain` is required for your workflow, reintroduce the CLI implementation and verify with `cargo build --bin sced_chain`.

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

### 3. Demonstration and Verification Path

The repository currently supports a demo and pilot verification path through [`src/bin/demo.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/demo.rs), [`src/bin/pilot_demo.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/pilot_demo.rs), and [`src/bin/verifier.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/verifier.rs).

It supports:

- deterministic demo scenarios
- generation of sample attestation logs
- JSON-based attestation verification
- clean operator stdout for the demo path

Primary commands:

```bash
cargo run --bin demo -- normal
cargo run --bin pilot_demo
cargo run --bin verifier pilot_attestation_log.json
```

Primary artifacts:

- [`pilot_attestation_log.json`](/workspaces/M.V.R.ESPRINT1/pilot_attestation_log.json)
- [`pilot_audit_ticket.md`](/workspaces/M.V.R.ESPRINT1/pilot_audit_ticket.md)

### 4. Demo and Harness Binaries

These binaries currently build and run:

- `demo`
- `dashboard`
- `formal_proof_harness`
- `pilot_demo`
- `verifier`
- `audit_ticket_verifier`

Their roles:

- `demo`: runs canned market-state scenarios through the demo pipeline
- `dashboard`: serves the demo dashboard, scenario API, and demo audit-ticket/report endpoints
- `formal_proof_harness`: checks simple invariant-style proofs against zero-state/TLBSS primitives
- `pilot_demo`: generates a sample attestation log and verifies it with `verifier`
- `verifier`: validates JSON attestation chains for `AttestationRecord`
- `sced_chain`: validates and benchmarks SCED offer-chain CSVs

## Repository Shape

Key library modules exported from [`src/lib.rs`](/workspaces/M.V.R.ESPRINT1/src/lib.rs):

- assurance and policy: `audit_guardian`, `regulatory_policy`, `constraint_system`, `compliance`
- simulation and harnessing: `simulation`, `simulation_harness_core`, `adversarial_harness`, `demo_pipeline`
- sovereign/runtime concepts: `sovereign_kernel`, `sovereign_trace`, `sovereign_bus`, `kernel`
- interfaces and drivers: `interface_discovery`, `protocol_drivers`, `operator_interface`, `drivers`, `sp_api`
- execution/codegen: `universal_frontend`, `ir_codegen`, `ir_backends`, `crypto_pipeline`
- SCED chain path: `sced_offer_chain`, `sced_decomposition`, `phase3_prediction`

## Recommended Commands

The current checkout does not include a Makefile or boot-script wrappers. Use the direct `cargo` binaries listed below.

Build the known-good binaries:

```bash
cargo build --bin verifier --bin demo --bin formal_proof_harness --bin dashboard --bin pilot_demo --bin audit_ticket_verifier
```

Run the library test suite:

```bash
cargo test --lib
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

Verify the generated attestation log:

```bash
cargo run --bin verifier pilot_attestation_log.json
```

Create and verify an audit ticket for this baseline:

```bash
cargo run --bin audit_ticket_verifier -- manifest-generate audit_manifest.json
cargo run --bin audit_ticket_verifier -- ticket-create audit_manifest.json audit_ticket.json "Baseline certification ticket"
cargo run --bin audit_ticket_verifier -- verify-ticket audit_ticket.json
```

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
