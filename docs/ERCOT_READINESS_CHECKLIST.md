# ERCOT Readiness Checklist

Use this checklist to move M.V.R.ESPRINT1 from prototype to submission-ready evidence.

## 1) Repository and Build Integrity

- [ ] CI passes on `main` for `fmt`, `clippy`, `check`, `test`
- [ ] Pinned Rust toolchain documented (`rust-toolchain.toml` or equivalent)
- [ ] `Cargo.lock` committed and reproducible builds verified
- [ ] Release build succeeds (`cargo build --release`)
- [ ] No `unsafe` blocks introduced (`#![deny(unsafe_code)]` remains enforced)

## 2) Determinism and Timing Evidence

- [ ] Deterministic replay of representative scenarios verified
- [ ] 1kHz loop timing bounds measured and documented
- [ ] Latency jitter envelope documented under expected load
- [ ] Time synchronization assumptions documented (PTP/NTP/clock source)
- [ ] Failure mode behavior under degraded timing captured

## 3) Safety and Guardrail Validation

- [ ] Unsafe transitions blocked by `audit_guardian` in tests
- [ ] Constraint violations produce explicit, auditable rejections
- [ ] L7 transitions require explicit external/operator authority
- [ ] No silent constraint relaxation paths exist
- [ ] Emergency behavior is deterministic and traceable

## 4) Security and Integrity

- [ ] Cryptographic hash chain validated end-to-end
- [ ] Signature and attestation verification tests pass
- [ ] Access control boundaries documented for all actors/interfaces
- [ ] Threat scenarios exercised in adversarial harness
- [ ] Dependency vulnerability scan completed and reviewed

## 5) Compliance Mapping Package

- [ ] BAL/PRC/FAC/CIP mapping table links code paths to obligations
- [ ] Each control has objective evidence (test, log, trace, artifact)
- [ ] Non-conformances tracked with mitigation and owner
- [ ] Regulatory assumptions clearly stated and versioned
- [ ] Scope boundaries stated (advisory vs assisted control)

## 6) Operational Readiness

- [ ] Deployment topology and rollback plan documented
- [ ] Runbook for startup/shutdown/recovery validated
- [ ] Incident response playbook includes trace retrieval
- [ ] Operator training walkthrough completed
- [ ] Change management process defined (who approves what)

## 7) Pilot and Submission Artifacts

- [ ] Pilot objective statement (what success looks like)
- [ ] Test matrix for normal/degraded/emergency scenarios
- [ ] Sample SovereignTrace bundle with verification steps
- [ ] Executive summary for non-technical reviewers
- [ ] Technical appendix with reproducible commands

## 8) Pre-Submission Gate

- [ ] All checklist items have status: Pass / Risk Accepted / Pending
- [ ] Open risks have owners and target dates
- [ ] Final evidence bundle reviewed end-to-end
- [ ] Submission package sign-off completed

## Suggested Evidence Folder Layout

```text
evidence/
  ci/
  deterministic-replay/
  timing/
  safety-guardrails/
  security-integrity/
  compliance-mapping/
  operations/
  pilot-results/
```

## Submission Notes

- Keep every claim tied to an artifact.
- Prefer reproducible command logs over screenshots.
- Present controls in reviewer language first, implementation details second.
