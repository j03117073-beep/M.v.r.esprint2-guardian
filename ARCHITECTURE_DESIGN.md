# M.V.R.ESPRINT1 Architecture Design

Updated to reflect the repository state verified on 2026-03-27.

## Executive Summary

M.V.R.ESPRINT1 currently has three operationally coherent layers:

- a deterministic SCED verification stack
- a hardened scenario kernel with auditable outputs
- an Integration Simulation Environment that replays realistic inputs into the kernel without embedding live transport

The system is designed so that external inputs are normalized and validated before they reach kernel execution.

## Architecture in Practice

### SCED Verification Stack

```text
CSV input
  -> parse_csv
  -> canonical sort
  -> primary-key validation
  -> record hash generation
  -> rolling chain-hash rebuild
  -> physics replay checks
  -> interval/event aggregation
  -> JSON report / benchmark output
```

Implementation:

- core logic: [`src/sced_offer_chain.rs`](/workspaces/M.V.R.ESPRINT1/src/sced_offer_chain.rs)
- CLI: [`src/bin/sced_chain.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/sced_chain.rs)

### Scenario Execution Stack

```text
Scenario Manifest
  -> dataset integrity checks
  -> ICCP adapter normalization
  -> external model validation
  -> unified state construction
  -> deterministic kernel execution
  -> attestation and audit export
```

Implementation:

- kernel: [`src/scenario_kernel.rs`](/workspaces/M.V.R.ESPRINT1/src/scenario_kernel.rs)
- attestation layer: [`src/sovereign_kernel.rs`](/workspaces/M.V.R.ESPRINT1/src/sovereign_kernel.rs)
- runner: [`src/bin/scenario_runner.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/scenario_runner.rs)
- failure classifier: [`src/failure_signal.rs`](/workspaces/M.V.R.ESPRINT1/src/failure_signal.rs)

### ISE Stack

```text
Replay
  -> interface simulation layer
  -> fixed snapshot handoff
  -> scenario kernel
  -> validation
  -> metrics and timeline reports
```

Implementation:

- core: [`src/ise.rs`](/workspaces/M.V.R.ESPRINT1/src/ise.rs)
- runner: [`src/bin/ise_runner.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/ise_runner.rs)

## Interface Boundary Design

The core architectural rule is that ICCP and model-generated inputs are external interfaces, not kernel internals.

### ICCP Boundary

[`src/iccp_adapter.rs`](/workspaces/M.V.R.ESPRINT1/src/iccp_adapter.rs) is responsible for:

- ingesting pre-decoded Block 1 and Block 2 data
- validating completeness and schema shape
- normalizing snapshots into kernel-compatible structures

The kernel does not implement:

- transport logic
- session handling
- streaming protocol behavior

### External Model Boundary

[`src/external_model_inputs.rs`](/workspaces/M.V.R.ESPRINT1/src/external_model_inputs.rs) is responsible for:

- accepting externally generated forecast or optimization inputs
- validating balance and feasibility
- rejecting internally inconsistent states

The kernel does not implement any statistical or predictive model.

## State Model

The unified scenario state now contains:

- core scenario load and outage data
- storage availability
- `telemetry_state`
- `control_context`
- external model interval inputs

All fields are time-aligned before execution. Invalid or incomplete alignment fails closed.

## Determinism Model

Determinism is preserved through the following rules:

- all external data is snapshotted per interval before execution
- identical snapshots must produce identical outputs
- ISE injections are applied before execution, not during replay
- no nondeterministic randomness exists in the replay path
- partial audit artifacts are not left behind on failure

## Failure Model

Failure is treated as a first-class output contract rather than an incidental log message.

Structured failure object:

```json
{
  "status": "INVALID",
  "failure_type": "SNAPSHOT_INCONSISTENCY",
  "invariant_violated": "STATE_INTEGRITY",
  "timestamp": "...",
  "execution_mode": "ISE_STEP"
}
```

This allows the architecture to support externally readable truth events for invalid runs.

## Current Library Groupings

The crate still exports a broader assurance-oriented library through [`src/lib.rs`](/workspaces/M.V.R.ESPRINT1/src/lib.rs). The most relevant current groupings are:

- verification and audit:
  - `sced_offer_chain`
  - `attestation_verifier`
  - `audit_exporter`
  - `failure_signal`
- scenario and integration execution:
  - `scenario_kernel`
  - `iccp_adapter`
  - `external_model_inputs`
  - `ise`
- broader experimental modules:
  - `simulation`
  - `simulation_harness_core`
  - `adversarial_harness`
  - `protocol_drivers`
  - `operator_interface`

## Verification Boundary

Implemented and verified:

- deterministic replay
- evidence export
- feature-gated attestation modes
- ICCP-aligned normalization
- external model input validation
- reusable integration replay with timeline logs

Outside current scope:

- actual ICCP protocol implementation
- full grid dynamics modeling
- live dispatch control
- embedded forecasting
