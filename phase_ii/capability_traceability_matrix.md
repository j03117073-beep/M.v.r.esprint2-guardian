# Capability Traceability Matrix

## Objective
Map documented operational claims to source implementation, verification method, evidence artifact, and failure mode.

| Capability | Source Implementation | Verification Method | Evidence Artifact | Failure Mode |
|---|---|---|---|---|
| `demo` scenario playback | `src/bin/demo.rs` | run `cargo run --bin demo -- normal` | `/tmp/phase_ii_repro/logs/demo_run.log` | runtime panic or invalid scenario output |
| `pilot_demo` attestation generation | `src/bin/pilot_demo.rs` | run `cargo run --bin pilot_demo` | `phase_ii/determinism/pilot_attestation_log_run_1.json` | missing artifact or inconsistent hashes |
| `verifier` attestation validation | `src/bin/verifier.rs` | run `cargo run --bin verifier pilot_attestation_log.json` | `/tmp/phase_ii_repro/logs/verifier_run.log` | invalid signature, chain break, parse error |
| `formal_proof_harness` invariant validation | `src/bin/formal_proof_harness.rs` | run `cargo run --bin formal_proof_harness` | `/tmp/phase_ii_repro/logs/formal_proof_harness_run.log` | invariant failure or runtime panic |
| `dashboard` server | `src/bin/dashboard.rs` | start server and observe `Server listening` | `/tmp/phase_ii_repro/logs/dashboard_run.log` | server startup failure, binding error |
| Attestation format contract | `src/sovereign_kernel.rs`, `src/bin/verifier.rs` | malformed input tests | `phase_ii/adversarial/*` | parse failures, schema field misses |
| Deterministic evidence artifact | `src/bin/pilot_demo.rs` | 10-run SHA256 comparison | `phase_ii/determinism/*.json` | differing SHA256 digest |
| `scenario_runner` CLI claim | none present in checkout | N/A | none | unsupported missing binary |
| `ise_runner` CLI claim | none present in checkout | N/A | none | unsupported missing binary |

## Conclusion
- The current executable claims are traceable to actual source and evidence.
- Documented runners `scenario_runner` and `ise_runner` are not present in this repository checkout, so those claims remain unsupported.

## Status
- PARTIAL
