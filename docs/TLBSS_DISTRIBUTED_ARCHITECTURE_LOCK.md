# TLBSS Distributed Architecture Lock

## Purpose
This document locks the deterministic implementation architecture for the tri-faction system:
- `M.V.R.E` (central control center)
- `sprint1` (deterministic binding substrate)
- `Guardian` (audit and integrity authority)

This lock is aligned to the canonical TLBSS master and empirical companion constraints.

## Canonical Role Mapping
- `Entity A` maps to `M.V.R.E` supervisory origin lane
- `Entity B` maps to `sprint1` cognitive mediation and deterministic binding lane
- `Entity C` maps to `Guardian` internalized non-agent audit lane

Primary flow remains:
- `A -> B -> C`

Secondary reflexive flow remains:
- `C -> B -> A'`

## Deterministic Control Spine
All runtime flows must follow this sequence:
1. ingest records in `sprint1`
2. enforce schema lock and normalization
3. deterministic sort by locked key
4. canonical serialization
5. record hash and chain hash construction
6. dual publish to `M.V.R.E` and `Guardian`
7. independent replay verification in `Guardian`
8. admissibility status exposure to `M.V.R.E`

No component may bypass steps 2 through 5.

## Locked Deterministic Rules
- Primary key:
  - `(scd_timestamp, repeat_hour_flag, resource_name, offer_type)`
- Sort order:
  1. `scd_timestamp` ascending
  2. `repeat_hour_flag` `false -> true`
  3. `resource_name` ascending
  4. `offer_type` ascending
- Canonical field order:
  1. `scd_timestamp`
  2. `repeat_hour_flag`
  3. `resource_name`
  4. `price1_urs ... quantity_mw6`
  5. `offer_type`
- Delimiter: `"|"`
- Hashing:
  - `record_hash = SHA256(canonical_record_string_utf8)`
  - `chain_hash = SHA256(previous_chain_hash + "|" + record_hash)`
  - genesis `previous_chain_hash = "0"`

## TLBSS Axiom Enforcement in Distributed Form
- `L1-L3`: implemented in deterministic parser, canonicalizer, and replay pipeline
- `L4-L5`: permitted only as bounded parameterized modulation layers, never as nondeterministic randomness in verifier paths
- `L6`: coherence certifies admissibility and boundary state
- `L7`: transition is allowed only after explicit boundary detection, never as implicit recursion

## Guardian Scope Lock
Guardian is audit-only:
- allowed:
  - replay
  - detect mismatch
  - report admissibility and failure codes
- forbidden:
  - generating new operational state
  - mutating canonical records
  - overriding `sprint1` canonical rules

If Guardian originates semantic state, the implementation is non-compliant.

## Distribution Model
- `sprint1` is authoritative for canonical record construction
- `M.V.R.E` consumes canonical stream for operations
- `Guardian` consumes identical canonical stream for independent verification
- `M.V.R.E` and `Guardian` do not canonicalize independently from raw input in production mode

## Visual Reference
See:
- `docs/MVRE_SPRINT1_GUARDIAN_VISUAL_ARCHITECTURE.md`

