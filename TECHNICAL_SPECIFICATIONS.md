# M.V.R.ESPRINT1 Technical Specifications

Updated to match the repository state verified on 2026-03-27.

## Scope

These specifications describe the currently implemented and verified deterministic paths:

- SCED verification and decomposition
- hardened scenario execution
- ICCP-aligned adapter ingestion
- external model input validation
- Integration Simulation Environment replay
- structured failure signaling

No predictive or transport-layer logic is embedded in the kernel core.

## Toolchain

- Rust edition: `2021`
- Package manifest: [`Cargo.toml`](/workspaces/M.V.R.ESPRINT1/Cargo.toml)
- Default attestation feature: `mock`
- Optional hardware attestation feature: `tpm`

Primary dependencies in active use include:

- `serde`
- `serde_json`
- `csv`
- `sha2`
- `hex`
- `ed25519-dalek`
- `anyhow`
- `tokio`
- `axum`
- `time`
- `tss-esapi` behind the optional `tpm` feature

## Verified Build Matrix

Passes:

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
cargo run --quiet --bin scenario_runner -- --mode pilot --json
cargo run --quiet --bin verifier -- scenario_attestation_log.json
cargo run --quiet --bin ise_runner -- --mode accelerated --factor 60
```

## SCED Verifier Specifications

### CLI Surface

Implemented in [`src/bin/sced_chain.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/sced_chain.rs):

```bash
cargo run --bin sced_chain -- verify <input.csv> [expected_hash]
cargo run --bin sced_chain -- verify-full <input.csv>
cargo run --bin sced_chain -- benchmark <input.csv>
cargo run --bin sced_chain -- verify-against <input.csv> <reference.csv>
cargo run --bin sced_chain -- decompose <zip_or_folder>
cargo run --bin sced_chain -- --decompose <zip_file>
cargo run --bin sced_chain -- predict --sample
cargo run --bin sced_chain -- predict --validate <prediction.json>
```

### Deterministic Processing Rules

- exact header matching
- canonical record sort by `scd_timestamp`, `repeat_hour_flag`, `resource_name`, `offer_type`
- numeric normalization to 6 decimals
- duplicate primary-key rejection
- chain-hash generation using SHA-256

### Physics Replay Rule

For extended-schema records:

```text
recomputed_lmp = SYSTEM_LAMBDA - (SHIFT_FACTOR * SHADOW_PRICE)
residual = abs(LMP - recomputed_lmp)
```

Failure threshold:

- `residual >= 0.001` triggers `MATH_DEVIATION`

## Scenario Kernel Specifications

### Files

- kernel: [`src/scenario_kernel.rs`](/workspaces/M.V.R.ESPRINT1/src/scenario_kernel.rs)
- attestation support: [`src/sovereign_kernel.rs`](/workspaces/M.V.R.ESPRINT1/src/sovereign_kernel.rs)
- CLI: [`src/bin/scenario_runner.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/scenario_runner.rs)
- failure classification: [`src/failure_signal.rs`](/workspaces/M.V.R.ESPRINT1/src/failure_signal.rs)

### Execution Flow

```text
Scenario Manifest
  -> dataset presence validation
  -> timestamp alignment validation
  -> ICCP adapter normalization
  -> external model input validation
  -> unified state construction
  -> deterministic kernel execution
  -> validation and attestation
  -> audit export
```

### CLI Surface

```bash
cargo run --bin scenario_runner -- --mode pilot|full [--manifest <path>] [--attestation-output <path>] [--audit-output <path>] [--quiet|--verbose] [--json]
```

### Output Contract

Primary outputs:

- [`scenario_attestation_log.json`](/workspaces/M.V.R.ESPRINT1/scenario_attestation_log.json)
- [`scenario_audit_ticket.md`](/workspaces/M.V.R.ESPRINT1/scenario_audit_ticket.md)

Standard stdout modes:

- default: clean operator-readable status lines
- `--quiet`: minimal output
- `--verbose`: expanded trace
- `--json`: machine-readable stdout

### State Vector Extensions

The unified state vector now includes:

- `telemetry_state`
- `control_context`

These are time-aligned and validated before execution.

### Audit Requirements

The Markdown audit ticket includes:

- scenario name
- execution timestamp
- input summary
- validation results
- determinism confirmation
- performance metrics
- ICCP snapshot reference
- timestamp alignment confirmation
- telemetry consistency validation
- external model snapshot reference

## ICCP Adapter Specifications

### Files

- adapter: [`src/iccp_adapter.rs`](/workspaces/M.V.R.ESPRINT1/src/iccp_adapter.rs)

### Responsibilities

- ingest pre-decoded ICCP-mapped data
- validate schema and completeness
- normalize into kernel-compatible snapshots

### Block Mapping

Block 1 maps to:

- real-time state inputs
- analog values
- system status indicators

Block 2 maps to:

- constraints
- schedules
- control intent signals

### Deterministic Rules

- ICCP data is snapshotted per SCED interval
- no live streaming occurs during kernel execution
- incomplete snapshots are rejected
- schema mismatch fails closed

## External Model Input Specifications

### Files

- validator: [`src/external_model_inputs.rs`](/workspaces/M.V.R.ESPRINT1/src/external_model_inputs.rs)

### Responsibilities

- accept externally generated forecast or optimization inputs
- validate feasibility against system constraints
- reject mathematically inconsistent states

### Integrity Checks

- load-generation balance
- constraint coherence
- storage feasibility
- temporal consistency across intervals

The kernel remains deterministic authority on correctness and does not implement any predictive model logic.

## ISE Specifications

### Files

- ISE core: [`src/ise.rs`](/workspaces/M.V.R.ESPRINT1/src/ise.rs)
- CLI: [`src/bin/ise_runner.rs`](/workspaces/M.V.R.ESPRINT1/src/bin/ise_runner.rs)

### Execution Flow

```text
Replay -> Interface Layer -> MVR Kernel -> Validation -> Metrics
```

### Supported Replay Modes

- `realtime`
- `accelerated`
- `step`

### Supported Injections

- `load-spike`
- `outage-stress`
- `constraint-stress`

### Determinism Guardrails

- injections are applied before execution by preparing a scenario variant
- source inputs are not mutated mid-run
- no nondeterministic randomness is used
- snapshot validation is not bypassed

### Reports

- [`ise_performance_report.json`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.json)
- [`ise_performance_report.md`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.md)
- [`ise_scenario_timeline_log.jsonl`](/workspaces/M.V.R.ESPRINT1/ise_scenario_timeline_log.jsonl)

## Structured Failure Signaling

All failure paths are classified into a machine-readable truth event via [`src/failure_signal.rs`](/workspaces/M.V.R.ESPRINT1/src/failure_signal.rs).

Canonical shape:

```json
{
  "status": "INVALID",
  "failure_type": "SNAPSHOT_INCONSISTENCY",
  "invariant_violated": "STATE_INTEGRITY",
  "timestamp": "...",
  "execution_mode": "ISE_STEP"
}
```

This object is emitted:

- on failing `scenario_runner --json` executions
- inside failing ISE JSON reports

## Current Known-Good Binaries

- `sced_chain`
- `verifier`
- `demo`
- `formal_proof_harness`
- `dashboard`
- `pilot_demo`
- `scenario_runner`
- `ise_runner`

## Boundaries

Implemented and verified:

- deterministic replay and validation
- evidence export
- feature-gated attestation mode selection
- ICCP-aligned snapshot ingestion
- external model input integrity checks
- reusable ISE replay

Not implemented:

- actual ICCP protocol transport
- live EMS or SCADA streaming integration
- full grid physics simulation
- statistical forecasting inside the kernel
- default TPM-backed attestation on every run
