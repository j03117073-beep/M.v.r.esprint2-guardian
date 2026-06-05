# Independent Reproduction Report

## Objective
Prove the documented workflow runs from a fresh checkout and toolchain environment.

## Procedure
- Copied repository contents to `/tmp/phase_ii_repro` excluding `target`, `.git`, and existing `phase_ii` artifacts.
- Built the repository binaries:
  - `verifier`
  - `demo`
  - `formal_proof_harness`
  - `dashboard`
  - `pilot_demo`
- Executed the documented runtime workflow:
  - `cargo run --bin demo -- normal`
  - `cargo run --bin pilot_demo`
  - `cargo run --bin verifier pilot_attestation_log.json`
  - `cargo run --bin formal_proof_harness`
  - `cargo run --bin dashboard`

## Results
- `demo`: success, output indicates stable and admissible scenario execution.
- `pilot_demo`: success, generated attestation artifact and printed verification guidance.
- `verifier`: success, verified generated attestation chain.
- `formal_proof_harness`: success, all invariants proven.
- `dashboard`: startup confirmed with server listening on `127.0.0.1:3000`.

## Evidence
- `/tmp/phase_ii_repro/logs/repro_build.log`
- `/tmp/phase_ii_repro/logs/demo_run.log`
- `/tmp/phase_ii_repro/logs/pilot_demo_run.log`
- `/tmp/phase_ii_repro/logs/verifier_run.log`
- `/tmp/phase_ii_repro/logs/formal_proof_harness_run.log`
- `/tmp/phase_ii_repro/logs/dashboard_run.log`

## Notes
- The dashboard startup was confirmed by log output, and the process was intentionally managed with a timeout wrapper in the reproduction flow.

## Status
- VERIFIED
