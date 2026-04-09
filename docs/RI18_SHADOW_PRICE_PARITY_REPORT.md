# RI_18 Shadow Price Parity Report

Date: April 8, 2026  
Scope: RTC+B shadow price parity checks against March 22 proxy data.

## Approach Chosen

Verification-first pass against proxy CSV:
- Implemented a deterministic parity checker that compares kernel shadow prices against proxy values.
- Enforces congestion sanity at 99% of limit and HALT threshold price uplift.
- Validates battery ECRS reservation so ECRS capacity is not misused as energy.

Module:
- `src/economics/shadow_prices.rs`

## Enforcement Rules

1. Parity tolerance: `abs(kernel - proxy) <= 1e-6` (configurable).
2. Congestion signal: if `flow >= 0.99 * limit`, shadow price must be non-trivial.
3. HALT mapping: if `flow >= halt_threshold` or `halt_triggered`, require price uplift.
4. Battery ECRS: `energy_used <= energy_available - ecrs_reserved`.

## Evidence (unit tests)

- `proxy_parity_and_congestion_checks_pass` -> PASS
- `flags_halt_threshold_without_price_uplift` -> FAIL expected (guard works)
- `flags_battery_ecrs_violation` -> FAIL expected (guard works)
- `march22_proxy_snapshot_parity_passes` -> PASS

## Pending for Full Parity Claim

## March 22 Proxy Snapshot

File:
- `data/ERCOT_SCED_PHYSICS_20260322_PROXY.csv`

SHA-256:
- `b07cca013e2bdc3e47f60d3f732df149fd15411bbbdf7e6e5e42821b7ceddee8`

Result:
- Parity check passed at tolerance `1e-6`.

Remaining gate for full external parity:
- Wire kernel shadow-price outputs into `ShadowPriceKernelRow` inputs for production data replay.
