# Coding Technical Framework

This framework reflects the repository state verified on 2026-03-27.

## Goals

- keep the deterministic verification paths stable
- keep the scenario and ISE evidence paths honest and reproducible
- ensure docs, tests, and artifacts match what actually passes

## Repository Priorities

### Priority 1: Keep Green Paths Green

The following commands are the current baseline and should remain green:

```bash
cargo check
cargo test --no-run
cargo test --lib
cargo test scenario_kernel --lib
cargo test external_model_inputs --lib
cargo test ise --lib
cargo build --bins
./scripts/boot_pilot_scenario.sh
./scripts/boot_full_scenario.sh
cargo run --quiet --bin pilot_demo
cargo run --quiet --bin verifier pilot_attestation_log.json
```

### Priority 2: Protect Deterministic Boundaries

High-signal files:

- [`src/sced_offer_chain.rs`](/workspaces/M.V.R.ESPRINT1/src/sced_offer_chain.rs)
- [`src/iccp_adapter.rs`](/workspaces/M.V.R.ESPRINT1/src/iccp_adapter.rs)
- [`src/external_model_inputs.rs`](/workspaces/M.V.R.ESPRINT1/src/external_model_inputs.rs)
- [`src/failure_signal.rs`](/workspaces/M.V.R.ESPRINT1/src/failure_signal.rs)
- [`src/bin/demo.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/demo.rs)
- [`src/bin/pilot_demo.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/pilot_demo.rs)
- [`src/bin/verifier.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/verifier.rs)

### Priority 3: Keep Evidence Contracts Stable

User-visible artifact contracts now include:

- [`scenario_attestation_log.json`](/workspaces/M.V.R.ESPRINT1/scenario_attestation_log.json)
- [`scenario_audit_ticket.md`](/workspaces/M.V.R.ESPRINT1/scenario_audit_ticket.md)
- [`ise_performance_report.json`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.json)
- [`ise_performance_report.md`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.md)
- structured failure objects on invalid runs

Changes to those shapes should be treated as contract changes and documented explicitly.

## Coding Standards

### Rust

- keep `#![deny(unsafe_code)]` intact
- prefer deterministic logic and explicit validation over convenience behavior
- keep external interfaces outside the kernel core
- classify failures instead of relying on ad hoc text errors
- avoid refactors that blur the boundary between replay, validation, and execution

### Documentation

- update docs whenever commands, reports, or evidence shapes change
- distinguish clearly between:
  - verified paths
  - experimental modules
  - out-of-scope capabilities

### Test Discipline

- put fast deterministic tests in library modules
- prefer snapshot-oriented tests for adapter and kernel behavior
- add failure-path coverage whenever a new invariant is introduced

## Recommended Validation Workflow

For scenario and validation changes:

```bash
cargo check
cargo test scenario_kernel --lib
cargo test external_model_inputs --lib
cargo run --quiet --bin pilot_demo
cargo run --quiet --bin verifier pilot_attestation_log.json
```

For ISE changes:

```bash
cargo test ise --lib
```

For failure classification changes:

```bash
cargo run --quiet --bin pilot_demo
cargo run --quiet --bin verifier pilot_attestation_log.json
```

## Determinism Guardrails

The following rules are now part of the engineering framework:

- no live mutation of replay inputs during a run
- no nondeterministic randomness in scenario or ISE execution
- no bypass of ICCP or external-model snapshot validation
- no partial audit artifacts on failure
- every invalid run must produce a structured failure signal

## Change Management

When making repo-wide changes:

1. confirm whether the change touches SCED, scenario, ICCP, external-model, ISE, or failure-contract paths
2. run the smallest relevant passing command set
3. update docs if user-visible behavior or artifact shape changed
4. preserve deterministic guarantees explicitly
5. refresh report examples if current artifact outputs changed

## Near-Term Engineering Recommendations

1. collect repeated baseline and stress-run metrics into a reusable ledger
2. exercise TPM-backed mode on actual hardware and document the evidence delta
3. expand the scenario matrix with more classified failure examples
4. keep the ISE focused on integration realism without drifting into full grid simulation
