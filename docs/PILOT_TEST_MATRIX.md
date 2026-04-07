# Pilot Test Matrix

Version: 2026-04-07.v1
Purpose: define the submission-ready test matrix for normal, degraded, and emergency scenarios with reproducible commands and expected results.

## Execution Notes

- Run from repository root.
- All commands are local CLI workflows.
- Store run logs under `evidence/pilot-results/` when producing submission artifacts.

## Normal Scenarios

| ID | Scenario | Command | Expected Result | Primary Evidence |
|---|---|---|---|---|
| N-01 | SCED deterministic verification (gold input) | `cargo run --bin sced_chain -- verify test_vectors/gold_truth_sced_20260322_1805.csv` | CLI emits `[PASS] verification_complete` and JSON report `status=PASS` | `src/bin/sced_chain.rs`, `src/sced_offer_chain.rs` |
| N-02 | Pilot attestation generation and verification | `cargo run --bin pilot_demo` | Generates `pilot_attestation_log.json` and verifier reports chain verified | `src/bin/pilot_demo.rs`, `src/bin/verifier.rs`, `pilot_attestation_log.json` |
| N-03 | Demo normal market scenario | `cargo run --bin demo -- normal` | No emergency path required; output reflects normal admissible flow | `src/bin/demo.rs`, `src/demo_pipeline.rs` |

## Degraded Scenarios

| ID | Scenario | Command | Expected Result | Primary Evidence |
|---|---|---|---|---|
| D-01 | Invalid boolean field in SCED CSV | `cargo run --bin sced_chain -- verify test_vectors/invalid_boolean.csv` | JSON report `status=FAIL` with code `INVALID_BOOLEAN` | `test_vectors/invalid_boolean.csv`, `src/bin/sced_chain.rs` |
| D-02 | Schema mismatch in SCED CSV | `cargo run --bin sced_chain -- verify test_vectors/schema_validation_error.csv` | JSON report `status=FAIL` with code `CSV_SCHEMA_MISMATCH` or `CSV_MALFORMED` | `test_vectors/schema_validation_error.csv`, verifier error mapping in `src/bin/sced_chain.rs` |
| D-03 | Hash mismatch against expected value | `cargo run --bin sced_chain -- verify test_vectors/fail_case_hash_mismatch.csv 0000000000000000000000000000000000000000000000000000000000000000` | JSON report `status=FAIL` with code `HASH_MISMATCH` | `test_vectors/fail_case_hash_mismatch.csv`, `src/sced_offer_chain.rs` |
| D-04 | Duplicate key handling around DST repeat-hour semantics | `cargo run --bin sced_chain -- verify test_vectors/dst_duplicate_without_flag.csv` | Duplicate key detected and verification fails with `DUPLICATE_PK` | `test_vectors/dst_duplicate_without_flag.csv`, duplicate-key checks in `src/sced_offer_chain.rs` |

## Emergency Scenarios

| ID | Scenario | Command | Expected Result | Primary Evidence |
|---|---|---|---|---|
| E-01 | Reserve shortage path | `cargo run --bin demo -- reserve` | Demo pipeline surfaces constrained/emergency mapping behavior for reserve scarcity | `src/bin/demo.rs`, `src/demo_pipeline.rs` |
| E-02 | Capacity shortage path | `cargo run --bin demo -- capacity` | Demo pipeline surfaces constrained/emergency mapping behavior for capacity scarcity | `src/bin/demo.rs`, `src/demo_pipeline.rs` |
| E-03 | Network overload path | `cargo run --bin demo -- network` | Demo pipeline surfaces constrained/emergency mapping behavior for network overload | `src/bin/demo.rs`, `src/demo_pipeline.rs` |
| E-04 | Collapse path | `cargo run --bin demo -- collapse` | Demo pipeline surfaces highest-severity emergency mapping behavior | `src/bin/demo.rs`, `src/demo_pipeline.rs` |
| E-05 | Adversarial emergency/guardrail validation suite | `cargo test --test adversarial_validation -- --nocapture` | Adversarial cases execute and report pass/fail against deterministic halt invariants | `tests/adversarial_validation.rs`, `src/testament_audit.rs` |

## Submission Artifact Checklist for This Matrix

- Capture stdout/stderr for each scenario run.
- Save command transcripts in `evidence/pilot-results/`.
- For failures in degraded scenarios, retain full JSON verifier output.
- For emergency scenarios, retain demo outputs that show scenario label and resulting mapped behavior.

## Deterministic Kernel Path Validation

| ID | Scenario | Command | Expected Result | Primary Evidence |
|---|---|---|---|---|
| K-01 | Normal path time invariant | `cargo test validation_matrix::tests::normal_path_valid_case_passes --lib` | Deterministic normal dispatch path passes with balanced-state checks | `src/validation_matrix.rs`, `docs/MARCH22_VALIDATION_MATRIX.md` |
| K-02 | Degraded stale/held telemetry substitute path | `cargo test validation_matrix::tests::degraded_path_uses_last_good_value_and_warns_err001 --lib` | `ERR_001`-equivalent warning emitted with last-good substitute behavior | `src/validation_matrix.rs`, `src/telemetry.rs` |
| K-03 | Emergency physics reserve-exhausted path | `cargo test validation_matrix::tests::emergency_physics_load_shed_exec_0x0c_when_reserves_exhausted --lib` | `EXEC_0x0C` emitted and load-shed action selected | `src/validation_matrix.rs`, `src/reliability_controls.rs` |
| K-04 | Emergency cyber sovereignty breach path | `cargo test validation_matrix::tests::emergency_cyber_halts_0x0b --lib` | `HALT_0x0B` deterministic secure lockdown on unauthorized write attempt | `src/validation_matrix.rs`, `src/reliability_controls.rs` |
