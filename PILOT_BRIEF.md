# M.V.R.ESPRINT1 Pilot Brief

Updated to match the working repository state on 2026-03-27.

## Executive Summary

M.V.R.ESPRINT1 currently demonstrates the strongest pilot value as an advisory-only deterministic validation and evidence system for SCED-style scenarios, integration replay, and operator-readable audit output.

The repo can already show:

- deterministic SCED normalization and hash chaining
- physics-consistent replay checks on extended SCED vectors
- scenario execution with human-readable audit tickets
- ICCP-aligned snapshot ingestion through an external adapter layer
- external model input validation without embedding predictive logic
- reusable ISE replay under normal and stressed conditions
- structured failure classification for invalid states

This makes the project suitable today for evaluation as a correctness authority and integration-readiness harness rather than as a full live deployment stack.

## Submission Framing

The strongest honest pilot and submission narrative is that M.V.R.ESPRINT1 is an offline, read-only, deterministic replay and analytical platform intended to support post-event reconstruction, engineering review, and evidence-based analysis. It is not designed to participate in AGC, market clearing, real-time dispatch, or any other live control loop.

That framing is important because it positions the project as a low-risk decision-support capability with clear auditability, reproducibility, and traceability rather than as a live operations system.

## Current Pilot Story

### What Is Real and Running

- `sced_chain` validates, benchmarks, and decomposes SCED-style vectors
- `scenario_runner` executes pilot and full scenario flows with deterministic audit outputs
- the scenario kernel validates ICCP snapshots, external model inputs, storage behavior, and timestamp alignment
- the ISE replays structured interface messages in `realtime`, `accelerated`, or `step` mode
- failure paths emit machine-readable classification objects instead of ambiguous exits

### What Is Not Yet Pilot-Ready

- live ICCP protocol transport
- live EMS or SCADA ingestion
- full grid physics simulation
- default hardware TPM evidence on every run

## Best Pilot Use Case Right Now

### Deterministic Replay, Validation, and Evidence Pack

Recommended evaluation flow:

1. load a repository-controlled scenario or SCED vector
2. validate schema, timestamps, ICCP blocks, and external model inputs
3. execute the deterministic kernel
4. emit operator-readable and machine-readable evidence
5. replay the same scenario through ISE to measure behavior under realistic integration conditions
6. inject controlled stress and observe classified failure behavior

Primary commands:

```bash
./scripts/boot_pilot_scenario.sh
cargo run --quiet --bin ise_runner -- --mode accelerated --factor 60
```

Stress command:

```bash
cargo run --quiet --bin ise_runner -- --mode step --inject load-spike --inject constraint-stress --json-output ise_stress_report.json --markdown-output ise_stress_report.md --timeline-output ise_stress_timeline.jsonl
```

## Current Pilot Assets

### Scenario and Replay Inputs

- [`scenarios/ercot_proxy_outage_stress_scenario_v1/scenario_manifest.json`](/workspaces/M.V.R.ESPRINT1/scenarios/ercot_proxy_outage_stress_scenario_v1/scenario_manifest.json)
- [`scenarios/ercot_proxy_outage_stress_scenario_v1/iccp_block1.json`](/workspaces/M.V.R.ESPRINT1/scenarios/ercot_proxy_outage_stress_scenario_v1/iccp_block1.json)
- [`scenarios/ercot_proxy_outage_stress_scenario_v1/iccp_block2.json`](/workspaces/M.V.R.ESPRINT1/scenarios/ercot_proxy_outage_stress_scenario_v1/iccp_block2.json)
- [`scenarios/ercot_proxy_outage_stress_scenario_v1/external_model_inputs.json`](/workspaces/M.V.R.ESPRINT1/scenarios/ercot_proxy_outage_stress_scenario_v1/external_model_inputs.json)
- [`test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv`](/workspaces/M.V.R.ESPRINT1/test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv)

### Current Evidence Outputs

- [`scenario_audit_ticket.md`](/workspaces/M.V.R.ESPRINT1/scenario_audit_ticket.md)
- [`scenario_attestation_log.json`](/workspaces/M.V.R.ESPRINT1/scenario_attestation_log.json)
- [`ise_performance_report.json`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.json)
- [`ise_stress_report.json`](/workspaces/M.V.R.ESPRINT1/ise_stress_report.json)

## Verified Build Envelope

Passes:

```bash
cargo check
cargo test --no-run
cargo test scenario_kernel --lib
cargo test external_model_inputs --lib
cargo test ise --lib
./scripts/boot_pilot_scenario.sh
./scripts/boot_full_scenario.sh
cargo run --quiet --bin ise_runner -- --mode accelerated --factor 60
```

## Pilot Positioning

Position the project today as:

- a deterministic verifier
- an operator-readable audit system
- an integration replay environment
- a correctness gate for external telemetry and model-generated inputs

Do not position it yet as:

- a live grid control platform
- a full ICCP transport implementation
- a predictive engine
- a full grid simulator

## Current Demonstration Claim

The strongest honest pilot claim is:

“Given fixed snapshots, MVR executes deterministically, explains its decisions clearly, and fails in a classified and reproducible way when system integrity breaks.”
