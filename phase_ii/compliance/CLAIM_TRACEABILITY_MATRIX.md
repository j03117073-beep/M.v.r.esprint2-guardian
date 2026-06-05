# Phase II Compliance Claim Traceability Matrix

## Purpose
Map every substantive claim in the repository to source code, verification method, evidence artifact, and status.

| Claim | Source Location | Verification Method | Evidence Artifact | Previous Status | State |
|---|---|---|---|---|---|
| `demo` scenario playback | `src/bin/demo.rs` | `cargo run --bin demo -- normal` | `phase_ii/artifacts/logs/demo_run.log` | VERIFIED | FAILED_VALIDATION |
| `pilot_demo` attestation generation | `src/bin/pilot_demo.rs` | `cargo run --bin pilot_demo` | `phase_ii/determinism/pilot_attestation_log_run_1.json` | VERIFIED | VALIDATED |
| `verifier` attestation validation | `src/bin/verifier.rs` | `cargo run --bin verifier phase_ii/evidence/pilot_attestation_log.json` | `phase_ii/adversarial/*`, `phase_ii/artifacts/logs/verifier_run.log` | VERIFIED | VALIDATED |
| `formal_proof_harness` invariant validation | `src/bin/formal_proof_harness.rs` | `cargo run --bin formal_proof_harness` | `phase_ii/artifacts/logs/formal_proof_harness_run.log` | VERIFIED | PENDING_EVIDENCE |
| SCED offer-chain verification | `src/sced_offer_chain.rs`, `src/bin/sced_chain.rs` | `cargo run --bin sced_chain -- verify <input.csv>` | `test_vectors/*`, `scripts/*` | PARTIAL | PENDING_EVIDENCE |
| Attestation format contract | `src/sovereign_kernel.rs`, `src/bin/verifier.rs` | malformed input tests | `phase_ii/adversarial/*` | VERIFIED | VALIDATED |
| Deterministic evidence artifact | `src/bin/pilot_demo.rs` | 10-run SHA256 comparison | `phase_ii/determinism/*.json` | VERIFIED | VALIDATED |
| `dashboard` server | `src/bin/dashboard.rs` | start server and observe `Server listening` | `phase_ii/artifacts/logs/dashboard_run.log` | VERIFIED | PENDING_EVIDENCE |
| `scenario_runner` CLI claim | none present | N/A | none | OPEN | INVALID_STRUCTURE |
| `ise_runner` CLI claim | none present | N/A | none | OPEN | INVALID_STRUCTURE |
| Phase III prediction payload sampling and validation | `README.md`, `OPERATIONAL_MANUAL.md` | review implementation and sample outputs | `test_vectors/*`, `scripts/*` | PARTIAL | PENDING_EVIDENCE |
| Independent audit readiness | `phase_ii/` reports + `phase_ii/compliance/` framework | review traceability and evidence completeness | `phase_ii/*`, `phase_ii/compliance/*` | PARTIAL | PENDING_EVIDENCE |

## Notes
- Claims with no source location are intentionally surfaced as OPEN.
- The matrix should be updated whenever a claim is added, changed, or moved.
- Future Phase III work must close `OPEN` items by either implementing them or explicitly marking them as unsupported.
