# Case Study #1 – March 22 ERCOT Event

This case study uses the repository’s verified March 22 ERCOT proxy vector and replay harness as a deterministic reconstruction exercise. It is intended as an engineering-style demonstration of MVRE Guardian’s offline analysis capability, not as a claim of a live ERCOT incident reconstruction.

## 1. What data was ingested?

The analysis ingested the repository’s March 22 ERCOT proxy dataset and the associated scenario inputs:

- the full-day proxy SCED/physics vector in [README.md](../README.md)
- a structured interval dataset containing 1,152 records across 288 intervals
- replay inputs from the ERCOT proxy outage stress scenario manifest
- telemetry/control-context snapshots and deterministic validation inputs used by the scenario kernel and ISE path

## 2. What did MVRE Guardian reconstruct?

MVRE Guardian reconstructed a deterministic timeline of the proxy event by:

- replaying the input stream in a fixed, reproducible order
- validating time alignment and snapshot completeness
- checking load-generation balance and constraint feasibility
- preserving an audit trail and attestation-style evidence package for review

In practice, this means the system reconstructed a technical state history of the scenario rather than a live operational control decision path.

## 3. What anomalies or state transitions did it identify?

In the verified stress replay, the system identified a classified state-integrity failure:

- failure type: SNAPSHOT_INCONSISTENCY
- invariant violated: STATE_INTEGRITY
- detection point: 2026-03-22 18:00:14
- outcome: the run was marked invalid and a reproducible failure record was preserved

This is a deterministic anomaly classification rather than a market-operations or operator-cause attribution.

## 4. How long did the analysis take?

The repository’s verified benchmark numbers show that the analysis was effectively sub-second:

- full-day proxy benchmark: 75 ms
- verified scenario path: 207 ms
- accelerated ISE replay over 96 intervals: 376 ms

## 5. How did its output compare with what ERCOT ultimately reported?

The output is narrower and more technical than an official ERCOT incident report. It is best understood as an engineering reconstruction layer that:

- confirms whether the scenario is internally consistent
- flags admissibility and state-integrity violations
- produces auditable evidence for post-event review

It does not attempt to replace ERCOT’s official public findings, which would normally include operator actions, market outcomes, and broader system-context explanations. The repository output is therefore best viewed as a deterministic decision-support and evidence-generation capability rather than a definitive public incident report.
