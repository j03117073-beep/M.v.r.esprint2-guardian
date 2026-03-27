# M.V.R.ESPRINT1 Performance Report

Verified against the repository state exercised on 2026-03-27.

## Executive Summary

This report records the currently verified performance envelope for the deterministic SCED path, the hardened scenario kernel, and the Integration Simulation Environment (ISE). The strongest operational path remains the repository-controlled ERCOT proxy scenario and verification vector:

- [`test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv`](/workspaces/M.V.R.ESPRINT1/test_vectors/ERCOT_SCED_PHYSICS_20260322_PROXY.csv)
- [`scenarios/ercot_proxy_outage_stress_scenario_v1/scenario_manifest.json`](/workspaces/M.V.R.ESPRINT1/scenarios/ercot_proxy_outage_stress_scenario_v1/scenario_manifest.json)

In the verified repository state:

- `cargo check` passed
- `cargo test --no-run` passed
- `cargo test scenario_kernel --lib` passed
- `cargo test external_model_inputs --lib` passed
- `cargo test ise --lib` passed
- `./scripts/boot_pilot_scenario.sh` passed
- `./scripts/boot_full_scenario.sh` passed
- `cargo run --quiet --bin ise_runner -- --mode accelerated --factor 60` passed

## Verified Baselines

### SCED Benchmark Baseline

Release benchmark result recorded in [`performance_ledger.json`](/workspaces/M.V.R.ESPRINT1/performance_ledger.json):

| Metric | Value |
| :--- | :--- |
| Build Hash | `34de832cc2964f4ef25aad6e751ca91b8a741ec8` |
| Mode | `full-scenario` |
| Scenario Name | `ERCOT_PROXY_OUTAGE_STRESS_SCENARIO_V1` |
| Benchmark Runtime | `75 ms` |
| Throughput | `15303 records/s` |
| Interval Throughput | `3826 intervals/s` |
| Scenario Runtime | `207 ms` |
| Scenario Intervals Processed | `96` |
| Scenario State Vectors/sec | `463` |

### ISE Accelerated Baseline

Baseline accelerated replay result from [`ise_performance_report.json`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.json):

| Metric | Value |
| :--- | :--- |
| Replay Mode | `accelerated` |
| Acceleration Factor | `60` |
| Intervals Replayed | `96` |
| Interface Messages Emitted | `288` |
| Execution Latency | `376 ms` |
| Records/sec | `321` |
| Validation Status | `PASS` |
| Determinism Verified | `true` |
| Execution Fingerprint | `bd079f7969469aaf77d217a11172b6989689d477e05263ff93f828c8fc50fcff` |

Associated outputs:

- [`ise_performance_report.json`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.json)
- [`ise_performance_report.md`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.md)
- [`ise_scenario_timeline_log.jsonl`](/workspaces/M.V.R.ESPRINT1/ise_scenario_timeline_log.jsonl)

## Stress Validation Outcome

The ISE was also exercised in `step` mode with deterministic stress injection:

```bash
cargo run --quiet --bin ise_runner -- --mode step --inject load-spike --inject constraint-stress --json-output ise_stress_report.json --markdown-output ise_stress_report.md --timeline-output ise_stress_timeline.jsonl
```

Observed outcome from [`ise_stress_report.json`](/workspaces/M.V.R.ESPRINT1/ise_stress_report.json):

- validation status: `FAIL`
- failure condition: `ICCP telemetry consistency validation failed at 2026-03-22 18:00:14`
- structured failure status: `INVALID`
- failure type: `SNAPSHOT_INCONSISTENCY`
- invariant violated: `STATE_INTEGRITY`
- execution mode: `ISE_STEP`

This is an expected and useful result. The stressed replay demonstrates that the system does not degrade silently under manipulated-but-plausible inputs. It classifies, halts, and preserves a reproducible failure record.

Associated outputs:

- [`ise_stress_report.json`](/workspaces/M.V.R.ESPRINT1/ise_stress_report.json)
- [`ise_stress_report.md`](/workspaces/M.V.R.ESPRINT1/ise_stress_report.md)
- [`ise_stress_timeline.jsonl`](/workspaces/M.V.R.ESPRINT1/ise_stress_timeline.jsonl)

## Methodology

### Scenario Path

The scenario path validates:

- dataset presence before execution
- timestamp alignment
- ICCP snapshot completeness
- external model input feasibility
- load-generation balance
- storage feasibility
- deterministic execution fingerprinting

Primary commands:

```bash
./scripts/boot_pilot_scenario.sh
./scripts/boot_full_scenario.sh
```

### ISE Path

The ISE replays fixed snapshots through a simulated ICCP-style interface layer. It does not:

- mutate source inputs mid-run
- introduce nondeterministic randomness
- bypass ICCP or external-model snapshot validation

Primary command:

```bash
cargo run --quiet --bin ise_runner -- --mode accelerated --factor 60
```

## Interpretation

The current numbers support the following claims:

- the SCED benchmark path is fast and repeatable on the repository-controlled proxy vector
- the hardened scenario kernel executes 96 intervals end to end with deterministic validation
- the ISE can replay realistic input snapshots and measure latency, throughput, validation behavior, and determinism
- failure behavior is externally readable through structured failure signals instead of ambiguous text-only exits

The current numbers do not support:

- live ISO production latency claims
- protocol-level ICCP transport performance claims
- full grid physics simulation claims
- statistical latency distribution claims across repeated timed campaigns

## Reproduction Steps

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

Optional failure-path reproduction:

```bash
cargo run --quiet --bin ise_runner -- --mode step --inject load-spike --inject constraint-stress --json-output ise_stress_report.json --markdown-output ise_stress_report.md --timeline-output ise_stress_timeline.jsonl
```

## Current Evidence Set

- [`scenario_attestation_log.json`](/workspaces/M.V.R.ESPRINT1/scenario_attestation_log.json)
- [`scenario_audit_ticket.md`](/workspaces/M.V.R.ESPRINT1/scenario_audit_ticket.md)
- [`performance_ledger.json`](/workspaces/M.V.R.ESPRINT1/performance_ledger.json)
- [`ise_performance_report.json`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.json)
- [`ise_performance_report.md`](/workspaces/M.V.R.ESPRINT1/ise_performance_report.md)
- [`ise_stress_report.json`](/workspaces/M.V.R.ESPRINT1/ise_stress_report.json)
- [`ise_stress_report.md`](/workspaces/M.V.R.ESPRINT1/ise_stress_report.md)

## Next Performance Work

1. Collect repeated benchmark runs and record min/median/max values.
2. Append repeated ISE baseline runs to a dedicated ledger alongside the scenario ledger.
3. Add a TPM-enabled attestation appendix once hardware-backed mode is exercised.
4. Track failure classifications over repeated stress scenarios to build a compact validation matrix.
