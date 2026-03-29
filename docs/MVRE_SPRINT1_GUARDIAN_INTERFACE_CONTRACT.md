# M.V.R.E / sprint1 / Guardian Interface Contract

## Purpose
Define the authoritative contract between:
- `M.V.R.E` (Central Control Center)
- `sprint1` (Deterministic Binding and Delivery Layer)
- `Guardian` (Audit and Integrity Authority)

This contract is mandatory for all distributed-control integrations.

## System Roles
- `M.V.R.E`
  - Supervisory control, dispatch, operational coordination
- `sprint1`
  - Deterministic normalization, schema lock enforcement, deterministic delivery
- `Guardian`
  - Independent replay verification, tamper detection, compliance evidence

## Trust Model
- `sprint1` is the only producer of canonical deterministic records consumed by both `M.V.R.E` and `Guardian`.
- `Guardian` must be able to verify all records without relying on `M.V.R.E` internal state.
- `M.V.R.E` may use records operationally, but cannot bypass canonicalization rules.

## Canonical Record Contract
### Sort Key (deterministic order)
1. `scd_timestamp` (ascending)
2. `repeat_hour_flag` (`false` then `true`)
3. `resource_name` (ascending)
4. `offer_type` (ascending)

### Primary Key
`(scd_timestamp, repeat_hour_flag, resource_name, offer_type)`

### Canonical Serialization Order
1. `scd_timestamp`
2. `repeat_hour_flag`
3. `resource_name`
4. price/quantity fields block 1..6 (fixed order)
5. `offer_type`

### Canonical Normalization Rules
- Trim all strings
- Numeric blank/null -> `0.000000`
- Numeric fixed precision -> 6 decimals
- Boolean accepted values -> `true`, `false`, `Y`, `N`
  - `Y` maps to `true`
  - `N` maps to `false`
- Reject all other boolean formats (`TRUE`, `1`, `0`, etc.)

## Hash and Chain Contract
- `record_hash = SHA256(canonical_record_string_utf8)`
- `chain_hash = SHA256(previous_chain_hash + "|" + record_hash)`
- Genesis value: `previous_chain_hash = "0"`

## Delivery Channels
### sprint1 -> M.V.R.E
- `topic`: `control.canonical_records`
- `payload`: canonical normalized SCED records
- `purpose`: operational decision support and command generation

### sprint1 -> Guardian
- `topic`: `audit.canonical_records`
- `payload`: same canonical normalized SCED records
- `purpose`: independent replay and integrity verification

### sprint1 -> Guardian (chain checkpoints)
- `topic`: `audit.chain_checkpoints`
- `payload`: chain metadata (`records_total`, `final_chain_hash`, version)
- `purpose`: fast integrity assertion and reconciliation

## Versioning
- `schema_version`: contract schema version (ex: `sced.v1`)
- `hash_spec_version`: hashing contract version (ex: `sha256.chain.v1`)
- Any change to field order, sort key, normalization, or delimiter requires version bump.

## Error Code Registry
- `DUPLICATE_PK`
- `INVALID_NUMERIC`
- `INVALID_BOOLEAN`
- `CSV_MALFORMED`
- `CSV_SCHEMA_MISMATCH`
- `RECORD_COUNT_MISMATCH`
- `CHAIN_CONTINUITY_BREAK`
- `HASH_MISMATCH`
- `LMP_MISMATCH` (optional sanity layer)

## Required Verifier JSON Output
```json
{
  "status": "PASS|FAIL",
  "records_total": 0,
  "records_verified": 0,
  "final_chain_hash": "hex",
  "expected_final_chain_hash": "hex|null",
  "mismatch_index": null,
  "errors": [
    {
      "code": "ERROR_CODE",
      "message": "human readable",
      "record_key": {
        "scd_timestamp": "string",
        "repeat_hour_flag": false,
        "resource_name": "string",
        "offer_type": "string"
      }
    }
  ]
}
```

## Operational Logs Contract
PASS path:
- `[INFO] verifier_start records=<N>`
- `[INFO] normalized_and_sorted`
- `[INFO] chain_rebuild_complete final_chain_hash=<hex>`
- `[PASS] verification_complete records_verified=<N>`

FAIL path:
- `[FAIL] code=<ERROR_CODE> mismatch_index=<i> key=(timestamp,flag,resource,offer)`

## Compliance Gate
Release is blocked unless:
- Canonical records are identical across replay runs for the same input
- Guardian replay matches sprint1 chain outputs
- All adversarial vector outcomes match expected fail/pass mapping
