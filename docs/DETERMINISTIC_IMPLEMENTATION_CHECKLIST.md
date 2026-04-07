# Deterministic Implementation Checklist

Last reviewed: April 7, 2026

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
- [ ] No cross-module state mutation that violates determinism

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
- [ ] optional expected `records_total` bound exposed through CLI argument

## E. Adversarial Vector Gate
- [ ] `gold_truth_sced_20260322_1805.csv` PASS
- [ ] `fail_case_hash_mismatch.csv` FAIL `HASH_MISMATCH`
- [ ] `schema_validation_error.csv` FAIL `CSV_SCHEMA_MISMATCH`
- [ ] `dst_duplicate_without_flag.csv` FAIL `DUPLICATE_PK`
- [ ] `dst_duplicate_with_flag.csv` PASS
- [ ] `invalid_boolean.csv` FAIL `INVALID_BOOLEAN`

## F. Compliance-Grade Release Gate
- [ ] `cargo check --lib` pass (Windows host blockers may apply)
- [ ] `cargo test --lib` pass (Windows host blockers may apply)
- [ ] deterministic replay consistency check (same input, same final hash, repeated)
- [ ] formal status directive updated with latest evidence

## April 2026 Status Update

- [x] Ubuntu WSL Rust toolchain installed (`rustc 1.94.1`, `cargo 1.94.1`)
- [x] Native Linux build dependencies installed (`build-essential`, `pkg-config`, `libssl-dev`)
- [x] `cargo check --message-format short` passes on Ubuntu 24.04 WSL
- [ ] `cargo test --lib` still pending in this verification pass
