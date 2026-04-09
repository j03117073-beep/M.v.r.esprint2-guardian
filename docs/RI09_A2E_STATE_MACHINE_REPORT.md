# RI_09 A2E State Machine Report

Date: April 8, 2026  
Scope: deterministic Approval-to-Energize guard for Sprint 1.

## Implementation

- Module: [`src/reliability/a2e_state_machine.rs`](C:\obienova\M.V.R.ESPRINT1\src\reliability\a2e_state_machine.rs)
- Export: [`src/reliability/mod.rs`](C:\obienova\M.V.R.ESPRINT1\src\reliability\mod.rs)

State model:
- `STAGED`
- `VALIDATING`
- `APPROVED`
- `REJECTED`

## Deterministic Guard Conditions

1. Temporal alignment:
- If `current_operating_day != energize_date`, decision remains `STAGED`.

2. Topology consistency:
- If equipment is absent from validated CIM set, decision is `REJECTED` with `HaltCip001`.

3. Predictive FAC safety:
- Uses threshold `min(1.25 * emergency_rating_2hr_mw, relay_loadability_mw if present)`.
- If `abs(predicted_post_close_flow_mw) > threshold`, decision is `REJECTED` with `HaltCip001`.

4. Approval:
- If all checks pass on energize day, decision is `APPROVED` and `a2e_permission_bit = 1`.

## Evidence

- `rejects_when_predicted_flow_is_1501_over_1500_threshold` -> PASS
- `approves_when_all_checks_pass` -> PASS
- `rejects_on_cim_mismatch_with_haltcip001` -> implemented in test suite
- `stays_staged_before_energize_day` -> implemented in test suite

Operational meaning:
- A 1501 MW predicted post-close flow against a 1500 MW threshold is deterministically blocked before breaker-close permission is granted.

