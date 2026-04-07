# ERCOT Telemetry Profile for Deterministic Validation

Version: 2026-04-07.v1
Scope: deterministic telemetry intake assumptions used by `src/telemetry.rs`.

## Ingestion Model

- Protocol framing target: ICCP-style pushed telemetry from participant systems.
- Push cadence target: 2-second operational scans for MW, voltage, and frequency points.
- Source-to-control-room latency target: less than 2 seconds.
- Staleness threshold: 10 seconds.

## Timestamp and Time Sync Assumptions

- Protocol-layer timestamps are treated as UTC in the deterministic validation model.
- Operational reporting may be rendered in CPT for operator-facing views.
- Time sync references: GPS disciplined timing using IRIG-B or PTP.
- Validation behavior:
  - mark stale when age exceeds 10 seconds
  - mark latency violation when source-to-ingest delay exceeds 2 seconds

## Quality Code Mapping

The profile uses these quality masks:

- `0x00`: valid
- `0x01`: held
- `0x02`: suspect
- `0x04`: manual

Current deterministic acceptance rule:

- only `0x00` is accepted as valid for strict deterministic validation
- held/suspect/manual values are flagged for explicit rejection paths

## Flow and Topology Signals

- Bus-oriented MW sign convention:
  - positive means flow out of bus
  - negative means flow into bus
- Tie-line validation uses actual flow values for physics checks.
- Breaker statuses support double-bit decoding:
  - `00` intermediate
  - `01` closed
  - `10` open
  - `11` bad state

## SCED Ramp and Dispatch Formulas Encoded

Implemented deterministic formulas:

- `SURAMP = min(normal_ramp_up, (HASL - current_output)/5)`
- `SDRAMP = min(normal_ramp_down, (current_output - LASL)/5)`
- `HDL = current_output + (SURAMP * 5)`
- `LDL = current_output - (SDRAMP * 5)`

Reserve cap rule encoded for thermal RRS:

- `RRS <= 20% * HSL`

## Deterministic Tie-Break Policies Encoded

- Economic ties at equal effective price are resolved with MW pro-rata allocation.
- Effective offer price respects mitigation caps when present:
  - `effective_price = min(offer_price, mitigated_offer_cap)`
- Deterministic epsilon overlay is supported for oscillation prevention:
  - lexical resource ordering with configurable epsilon increment.
- QSGR-style safeguard:
  - tiny dispatch needs do not force offline starts when online tied units can satisfy demand.
- RTC-style reserve interaction:
  - reserve-locked MW is removed from energy tie-break room before pro-rata allocation.
- Transmission tie curtailment:
  - pro-rata curtailment path is reused for equal-impact tie conditions.

## Source Files

- `src/telemetry.rs`
- `src/drivers/ptp_clock.rs`
- `src/constraint_system.rs` (constraint evaluation context)
