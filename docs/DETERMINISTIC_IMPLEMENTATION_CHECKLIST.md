# Deterministic Implementation Checklist

Last reviewed: April 8, 2026

## Purpose
Execution checklist for completing deterministic tri-faction integration in this order:
1. architecture lock
2. module boundaries
3. verifier consistency
4. test evidence
5. release gate

## A. Architecture Lock
- [x] TLBSS mapping to `M.V.R.E / sprint1 / Guardian` documented
- [x] Interface contract documented
- [x] Visual architecture documented
- [x] Architecture docs referenced from top-level `README.md`

## B. Faction Module Boundaries
- [x] `src/mvre.rs` created
- [x] `src/sprint1.rs` created
- [x] `src/guardian.rs` created
- [x] `src/lib.rs` exports all three modules
- [x] No cross-module state mutation that violates determinism

## C. Deterministic Data Rules
- [x] PK lock: `(scd_timestamp, repeat_hour_flag, resource_name, offer_type)`
- [x] deterministic sort lock enforced
- [x] canonical serialization order enforced
- [x] delimiter lock (`|`) enforced
- [x] fixed float normalization (`6` decimals)
- [x] explicit duplicate key detection
- [x] explicit chain continuity validation
- [x] empty file behavior locked to genesis hash `0`

## D. Verifier Contract
- [x] JSON status contract present
- [x] machine-readable error codes present
- [x] `mismatch_index` defined as 0-based post-sort
- [x] logs emit PASS and FAIL path lines
- [x] optional expected `records_total` bound exposed through CLI argument

## E. Adversarial Vector Gate
- [x] `gold_truth_sced_20260322_1805.csv` PASS
- [x] `fail_case_hash_mismatch.csv` FAIL `HASH_MISMATCH`
- [x] `schema_validation_error.csv` FAIL `CSV_SCHEMA_MISMATCH`
- [x] `dst_duplicate_without_flag.csv` FAIL `DUPLICATE_PK`
- [x] `dst_duplicate_with_flag.csv` PASS
- [x] `invalid_boolean.csv` FAIL `INVALID_BOOLEAN`

## F. Compliance-Grade Release Gate
- [x] `cargo check --lib` pass (Windows host blockers may apply)
- [x] `cargo test --lib` pass (Windows host blockers may apply)
- [x] deterministic replay consistency check (same input, same final hash, repeated)
- [x] formal status directive updated with latest evidence

## April 2026 Status Update

- [x] Ubuntu WSL Rust toolchain installed (`rustc 1.94.1`, `cargo 1.94.1`)
- [x] Native Linux build dependencies installed (`build-essential`, `pkg-config`, `libssl-dev`)
- [x] `cargo check --message-format short` passes on Ubuntu 24.04 WSL
- [x] `cargo test --lib` passes in this verification pass

## April 8, 2026 Evidence Notes

- `cargo run --bin sced_chain -- verify test_vectors/gold_truth_sced_20260322_1805.csv` -> PASS
- `cargo run --bin sced_chain -- verify test_vectors/fail_case_hash_mismatch.csv 000...000` -> FAIL `HASH_MISMATCH`
- `cargo run --bin sced_chain -- verify test_vectors/schema_validation_error.csv` -> FAIL `CSV_SCHEMA_MISMATCH`
- `cargo run --bin sced_chain -- verify test_vectors/dst_duplicate_without_flag.csv` -> FAIL `DUPLICATE_PK`
- `cargo run --bin sced_chain -- verify test_vectors/dst_duplicate_with_flag.csv` -> PASS
- `cargo run --bin sced_chain -- verify test_vectors/invalid_boolean.csv` -> FAIL `INVALID_BOOLEAN`
- `cargo run --bin sced_chain -- verify test_vectors/gold_truth_sced_20260322_1805.csv --records-total 3` -> PASS
- `cargo run --bin sced_chain -- verify test_vectors/gold_truth_sced_20260322_1805.csv --records-total 2` -> FAIL `RECORD_COUNT_MISMATCH`
- repeated replay hash checks produced identical `final_chain_hash` (`62d0da44f5d4294fcf1d69fee936547c0bf4b3ae86fc04d46fbd99aaf9c4d71b`)
