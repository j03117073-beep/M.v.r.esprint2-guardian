# Phase II Determinism Report v1

## Objective
Prove that `pilot_demo` produces repeatable attestation outputs.

## Workflow
Executed `cargo run --quiet --bin pilot_demo` 10 times in the current repository checkout and preserved each generated artifact under `phase_ii/determinism`.

## Outcome
- Generated artifacts: `phase_ii/determinism/pilot_attestation_log_run_{1..10}.json`
- SHA256 digest for all 10 files:
  - `4e93d35b8812727e9faed3dab79227c3473ca19c72c866fee8b4a33e757cc2c3`

## Conclusion
- All 10 runs produced byte-for-byte identical attestation logs.
- No divergence was observed.

## Evidence
- `phase_ii/determinism/pilot_attestation_log_run_1.json`
- `phase_ii/determinism/pilot_attestation_log_run_10.json`
- `phase_ii/determinism/pilot_attestation_log_run_2.json`
- `phase_ii/determinism/pilot_attestation_log_run_3.json`
- `phase_ii/determinism/pilot_attestation_log_run_4.json`
- `phase_ii/determinism/pilot_attestation_log_run_5.json`
- `phase_ii/determinism/pilot_attestation_log_run_6.json`
- `phase_ii/determinism/pilot_attestation_log_run_7.json`
- `phase_ii/determinism/pilot_attestation_log_run_8.json`
- `phase_ii/determinism/pilot_attestation_log_run_9.json`

## Status
- VERIFIED
