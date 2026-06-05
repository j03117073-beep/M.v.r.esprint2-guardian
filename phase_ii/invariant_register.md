# Invariant Register v1

## Invariant 1: Pilot Attestation Determinism
- Description: `pilot_demo` must emit identical attestation logs on repeated runs.
- Enforcement Mechanism: deterministic `SovereignKernel` simulation with fixed-key signer and fixed PCR semantics.
- Evidence Artifact: `phase_ii/determinism/pilot_attestation_log_run_1.json` through `pilot_attestation_log_run_10.json`
- Verification Method: SHA256 digest comparison across runs.
- Status: VERIFIED

## Invariant 2: Attestation Chain Integrity
- Description: each record must link to the previous record through `prev_hash` and maintain a zero initial chain anchor.
- Enforcement Mechanism: `verifier` chain linkage check in `src/bin/verifier.rs`.
- Evidence Artifact: `phase_ii/adversarial/invalid_prev_hash.json`
- Verification Method: rejection on index mismatch by `verifier`.
- Status: VERIFIED

## Invariant 3: Signature Validity
- Description: `signature` must equal `SHA256(decision_hash || pcr_digest)`.
- Enforcement Mechanism: `verifier` signature check in `src/bin/verifier.rs`.
- Evidence Artifact: `phase_ii/adversarial/invalid_signature.json`
- Verification Method: `verifier` rejects invalid signature payloads.
- Status: VERIFIED

## Invariant 4: Timestamp Monotonicity
- Description: attestation timestamps must be non-decreasing across the chain.
- Enforcement Mechanism: `verifier` timestamp ordering check in `src/bin/verifier.rs`.
- Evidence Artifact: `phase_ii/adversarial/stale_timestamp.json`
- Verification Method: `verifier` rejects backward timestamp progression.
- Status: VERIFIED

## Invariant 5: Evidence Format Contract
- Description: verifier input must be a JSON array of `AttestationRecord` objects.
- Enforcement Mechanism: `serde_json::from_str` parse on `src/bin/verifier.rs`.
- Evidence Artifacts: `phase_ii/adversarial/empty.json`, `invalid_json.json`, `prefixed_invalid.json`, `truncated.json`
- Verification Method: parser rejects malformed or non-array input.
- Status: VERIFIED
