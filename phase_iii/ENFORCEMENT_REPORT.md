# Enforcement Report (Tool)

Generated: /workspaces/M.v.r.esprint1-g

Claims scanned: 12
Evidence artifacts scanned: 14

## Claim Linkage Summary

- demo scenario playback: linked evidence count: 1
  - Demo run log (non-reproducible (external/tmp))
- pilot_demo attestation generation: linked evidence count: 1
  - Pilot attestation logs (reproducible (repo-local))
- verifier attestation validation: linked evidence count: 3
  - Pilot attestation logs (reproducible (repo-local))
  - Verifier run log (non-reproducible (external/tmp))
  - Verifier failure matrix (reproducible (repo-local))
- formal_proof_harness invariant validation: linked evidence count: 1
  - Invariant register (reproducible (repo-local))
- SCED offer-chain verification: NO LINKED EVIDENCE → PENDING_EVIDENCE
- Attestation format contract: linked evidence count: 1
  - Adversarial invalid payloads (reproducible (repo-local))
- Deterministic evidence artifact: linked evidence count: 3
  - Pilot attestation logs (reproducible (repo-local))
  - Determinism artifacts (reproducible (repo-local))
  - Determinism report (reproducible (repo-local))
- dashboard server: NO LINKED EVIDENCE → PENDING_EVIDENCE
- scenario_runner CLI claim: NO LINKED EVIDENCE → PENDING_EVIDENCE
- ise_runner CLI claim: NO LINKED EVIDENCE → PENDING_EVIDENCE
- Phase III prediction payload sampling and validation: NO LINKED EVIDENCE → PENDING_EVIDENCE
- Independent audit readiness: linked evidence count: 3
  - Branch analysis reports (reproducible (repo-local))
  - Integration plan (reproducible (repo-local))
  - Readiness assessment (reproducible (repo-local))

## Summary

- Validated claims (linked): 7/12
- Pending claims (unlinked): 5/12

## Unlinked Claims

- SCED offer-chain verification
- dashboard server
- scenario_runner CLI claim
- ise_runner CLI claim
- Phase III prediction payload sampling and validation

## Evidence Reproducibility Audit

Reproducible (repo-local): 10
- Pilot attestation logs
- Adversarial invalid payloads
- Determinism artifacts
- Branch analysis reports
- Integration plan
- Readiness assessment
- Determinism report
- Capability matrix
- Invariant register
- Verifier failure matrix

Non-reproducible (external/tmp): 4
- Demo run log
- Verifier run log
- Formal proof harness log
- Reproduction logs
