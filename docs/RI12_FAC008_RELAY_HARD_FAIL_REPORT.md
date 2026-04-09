# RI_12 FAC-008 Relay Hard-Fail Report

Date: April 8, 2026  
Scope: deterministic relay loadability enforcement for Sprint 1.

## Implementation

- New module: [`src/reliability/relay_logic.rs`](C:\obienova\M.V.R.ESPRINT1\src\reliability\relay_logic.rs)
- Exported via: [`src/reliability/mod.rs`](C:\obienova\M.V.R.ESPRINT1\src\reliability\mod.rs)

Core function:
- `evaluate_relay_hard_fail_dc(graph, branch_flow_mw, profiles)`

Enforcement rule:
- Threshold = `min(1.25 * emergency_rating_2hr_mw, relay_loadability_mw if present)`
- If `abs(flow_mw) > threshold`:
  - halt SCED cycle
  - emit code `HALT_0x0C`
  - trip violating branch(es) from topology graph

## Evidence Scenario (L1)

Input:
- Normal rating: `1000 MW`
- Emergency 2-hour rating: `1200 MW`
- 125% threshold: `1500 MW`
- Simulated flow: `1501 MW`

Expected:
- Hard-fail and branch trip.

Automated test:
- `halts_and_trips_when_flow_exceeds_125_percent_emergency_rating`

Boundary test:
- `does_not_halt_at_exact_125_percent_boundary`
- Confirms `1500 MW` does not trip (strict greater-than policy).

## Notes

- First pass uses deterministic DC-style branch flow input (`branch_flow_mw`) for enforcement.
- Next enhancement can bind solved AC branch flows (`I = Y * V`) once full solver integration is wired.

