# ERCOT Readiness Checklist

Use this checklist to move M.V.R.ESPRINT1 from prototype to submission-ready evidence.

Legend: `PASS` = completed and evidenced in-repo as of April 7, 2026.

## 1) Repository and Build Integrity

- [ ] CI passes on `main` for `fmt`, `clippy`, `check`, `test`
- [x] PASS Pinned Rust toolchain documented (`rust-toolchain.toml` or equivalent)
- [ ] `Cargo.lock` committed and reproducible builds verified
- [ ] Release build succeeds (`cargo build --release`)
- [x] PASS No `unsafe` blocks introduced (`#![deny(unsafe_code)]` remains enforced)

## 2) Determinism and Timing Evidence

- [ ] Deterministic replay of representative scenarios verified
- [ ] 1kHz loop timing bounds measured and documented
- [ ] Latency jitter envelope documented under expected load
- [x] PASS Time synchronization assumptions documented (`docs/ERCOT_TELEMETRY_PROFILE.md`)
- [x] PASS Failure mode behavior under degraded timing captured (`src/telemetry.rs` tests)

## 3) Safety and Guardrail Validation

- [ ] Unsafe transitions blocked by `audit_guardian` in tests
- [ ] Constraint violations produce explicit, auditable rejections
- [ ] L7 transitions require explicit external/operator authority
- [ ] No silent constraint relaxation paths exist
- [ ] Emergency behavior is deterministic and traceable

## 4) Security and Integrity

- [ ] Cryptographic hash chain validated end-to-end
- [ ] Signature and attestation verification tests pass
- [x] PASS Access control boundaries documented for all actors/interfaces
- [x] PASS Threat scenarios exercised in adversarial harness
- [ ] Dependency vulnerability scan completed and reviewed

## 5) Compliance Mapping Package

- [x] PASS BAL/PRC/FAC/CIP mapping table links code paths to obligations
- [x] PASS Each control has objective evidence (test, log, trace, artifact)
- [x] PASS Non-conformances tracked with mitigation and owner
- [x] PASS Regulatory assumptions clearly stated and versioned
- [x] PASS Scope boundaries stated (advisory vs assisted control)

## 6) Operational Readiness

- [ ] Deployment topology and rollback plan documented
- [ ] Runbook for startup/shutdown/recovery validated
- [ ] Incident response playbook includes trace retrieval
- [ ] Operator training walkthrough completed
- [ ] Change management process defined

## 7) Pilot and Submission Artifacts

- [x] PASS Pilot objective statement
- [x] PASS Test matrix for normal/degraded/emergency scenarios (`docs/PILOT_TEST_MATRIX.md`)
- [x] PASS Sample SovereignTrace bundle with verification steps
- [x] PASS Executive summary for non-technical reviewers (`docs/EXECUTIVE_SUMMARY_NON_TECHNICAL.md`)
- [x] PASS Performance summary linked (`PERFORMANCE_REPORT.md`)
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

## April 2026 Evidence Update

- [x] PASS Ubuntu 24.04 WSL development environment documented with required packages
- [x] PASS Workspace build verification command documented as `cargo check --message-format short`
- [x] PASS Compliance mapping and access-control matrix documented in `docs/compliance_mapping.md`
- [ ] Release build and binary smoke test evidence still need a clean post-restart WSL run
