# March 22, 2026 Deterministic Validation Matrix

Version: 2026-04-07.v1
Scope: strict normal/degraded/emergency expected behavior for sprint kernel validation.

## Path Definitions

### 1) Normal Path - Steady State Dispatch

- Preconditions:
  - telemetry quality valid (`0x00`)
  - time-sync delta less than or equal to 50 ms for strict deterministic clock alignment
  - resource outputs stay within LSL/HSL
  - base-point deltas respect SURAMP/SDRAMP
  - power balance closes:
    - `sum_generation + net_interchanges = load + losses`
- Deterministic action:
  - execute dispatch path (`EXECUTE_SCED`) with alignment buffering as needed.
- Fail condition:
  - timing invariant breach maps to `HALT_0x0A`.
  - physics/ramp violations fail the normal path deterministically.

### 2) Degraded Path - Telemetry/Link Failure

- Preconditions:
  - stale/held/suspect/manual telemetry or latency/staleness threshold breach.
- Deterministic action:
  - enter substitute-value path (`USE_STATE_ESTIMATOR`) using last-known-good value.
  - emit `ERR_001` equivalent (`Err001StaleSubstitute` in code path).
- Safety requirement:
  - execution loop must continue without blocking on missing I/O.

### 3) Emergency Path - Physics/Cyber

- Physics triggers:
  - UFRT condition (`frequency < 59.4 Hz` for more than 9 cycles)
  - FAC-008 emergency thermal overrun conditions
- Cyber triggers:
  - unauthorized base-point write attempts or service boundary violations.
- Deterministic actions:
  - `PRC_024_TRIP`/panic dispatch/load shed for physics emergencies.
  - secure fail-safe halt for cyber breaches.
- Codes:
  - `EXEC_0x0C` when reserves are exhausted and load shed is required.
  - `HALT_0x0B` for sovereignty breach / unauthorized write attempts.

## Implementation Pointers

- `src/validation_matrix.rs`
- `src/reliability_controls.rs`
- `src/telemetry.rs`

## Logic Validation Mapping

| Condition | Path | Kernel Logic Action | Expected Output State |
|---|---|---|---|
| Telemetry OK | Normal | `EXECUTE_SCED` / `BUFFER_ALIGN` | Balanced grid, deterministic dispatch |
| 1-2s jitter | Normal | `BUFFER_ALIGN` | Balanced grid (latency masked) |
| Missing MW point | Degraded | `USE_STATE_ESTIMATOR` | Substituted value, warning code |
| Frequency < 59.4 Hz | Emergency | `PRC_024_TRIP` / mitigation | Resource offline or load shed |
| Invalid memory access | Emergency | `KERNEL_PANIC / HALT` | Secure lockdown |

## EEA Context for March 22, 2026

- For historical replay validation, `EXEC_0x0C` should only occur if:
  - reserves are exhausted in-model, or
  - faults are intentionally injected for emergency-path testing.

