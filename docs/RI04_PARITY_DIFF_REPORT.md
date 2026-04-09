# RI_04 Parity-Diff Report

Date: April 8, 2026  
Scope: deterministic `Ybus` construction hard-gate for Sprint 1.

## Implemented Components

- Raw topology ingest path in [`src/ingest/rdf_parser.rs`](C:\obienova\M.V.R.ESPRINT1\src\ingest\rdf_parser.rs)
- Deterministic graph assembly in [`src/topology/graph_builder.rs`](C:\obienova\M.V.R.ESPRINT1\src\topology\graph_builder.rs)
- Sparse `Ybus` builder and parity diff logic in [`src/topology/ybus.rs`](C:\obienova\M.V.R.ESPRINT1\src\topology\ybus.rs)

## Hard-Gates Verified

1. Zero-Impedance Branch handling:
`topology::ybus::tests::zero_impedance_branch_uses_penalty_conductance` passed.
- Uses penalty conductance `G = 1_000_000` when `|Z|^2 <= 1e-12`.
- Captures affected branches in `zib_penalty_branches`.

2. Shunt charging (`Bch/2`) diagonal stamping:
`topology::ybus::tests::stamps_series_and_half_shunt` passed.
- Applies `+j(Bch/2)` to each terminal diagonal.
- Off-diagonal terms stamped as `-Yseries`.

3. Deterministic sparse parity threshold:
`topology::ybus::tests::parity_diff_hard_fails_over_threshold` passed.
- Threshold used: `1e-7`.
- Case A: perturbation `5e-8` -> PASS.
- Case B: perturbation `2.5e-7` -> FAIL.

## Current Evidence Outputs

- `cargo test --lib topology::ybus::tests::stamps_series_and_half_shunt` -> PASS
- `cargo test --lib topology::ybus::tests::zero_impedance_branch_uses_penalty_conductance` -> PASS
- `cargo test --lib topology::ybus::tests::parity_diff_hard_fails_over_threshold` -> PASS
- `cargo check --lib` -> PASS

## RI_04 Closure Status

Status: PASS for March 22 proxy target rows (engineering gate met).

## March 22 Proxy Row Results

Validated via:
- `topology::ybus::tests::march22_proxy_snapshot_hits_1e_7_mark`
- `topology::graph_builder::tests::telemetry_override_splits_bus_when_breaker_is_open`

Measured error metrics for provided proxy targets:
- `max_abs_error = 9.90098669717554e-10`
- `mae = 3.63036230706844e-10`
- tolerance = `1e-7`
- Result: PASS (`max_abs_error < 1e-7`)

RI_09 split verification (operational topology mode):
- Telemetry override now splits `CN_A` and `CN_B` when breaker telemetry is open despite modeled closed state.
- This behavior is covered by the `telemetry_override_splits_bus_when_breaker_is_open` test.

Remaining gate for full external closure:
- Re-run parity against the full March 22 operating-day sparse snapshot package when released and archive artifacts in `evidence/deterministic-replay/`.
