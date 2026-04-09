# Compliance Mapping and Access Control Matrix

Version: 2026-04-07.v1
Repository: `M.V.R.ESPRINT1`
Purpose: link BAL/PRC/FAC/CIP obligations to code paths and objective evidence while documenting actor/interface access boundaries.

## 1) Scope Boundary (Advisory vs Assisted Control)

- Current implemented and documented mode: advisory and verification-first workflows.
- In-scope binaries for this phase: `pilot_demo`, `verifier`, `sced_chain`, `demo`, `dashboard`, `formal_proof_harness`.
- Out-of-scope for current submission evidence: autonomous closed-loop dispatch authority.
- Assisted-control language in design docs is roadmap-only until a separate control-authority gate package is approved.

## 2) Access Control Boundaries (Actors and Interfaces)

| Actor | Allowed Interfaces | Allowed Actions | Explicitly Forbidden Actions | Evidence |
|---|---|---|---|---|
| `sprint1` deterministic substrate | `src/sprint1.rs`, `src/sced_offer_chain.rs` | Parse, schema-lock, deterministic sort, canonicalize, hash-chain build, publish canonical batch | Silent schema mutation, nondeterministic reordering, bypass of canonicalization sequence | `docs/TLBSS_DISTRIBUTED_ARCHITECTURE_LOCK.md`, unit tests in `src/sced_offer_chain.rs` |
| `Guardian` audit authority | `src/guardian.rs` | Replay and verify canonical batches, report admissibility, detect mismatches | Generating operational state, mutating canonical records, overriding `sprint1` rules | `src/guardian.rs` tests, `docs/TLBSS_DISTRIBUTED_ARCHITECTURE_LOCK.md` |
| `M.V.R.E` operational consumer | `src/mvre.rs` | Consume canonical outputs, track last seen chain hash, present operator-visible state | Re-canonicalizing raw production input independently, mutating authoritative canonical payloads | `src/mvre.rs`, architecture lock document |
| Operator-facing tools | `src/bin/*.rs`, `dashboard.html` | Execute local verification workflows and inspect outputs | Implicit authority escalation without explicit operator action | `OPERATIONAL_MANUAL.md`, `TECHNICAL_SPECIFICATIONS.md` |

## 3) BAL/PRC/FAC/CIP Mapping to Code Paths and Evidence

| Control Family | Obligation Framing | Code Paths | Objective Evidence | Status |
|---|---|---|---|---|
| BAL-001/002 | Deterministic replay and frequency-event reconstruction support | `src/sovereign_kernel.rs`, `src/demo_pipeline.rs`, `src/constraint_system.rs` | `pilot_attestation_log.json`, `src/bin/pilot_demo.rs`, `PILOT_BRIEF.md` | Pass (design + artifact) |
| PRC-005/023 | Protection and boundary-condition validation with explicit guardrails | `src/audit_guardian.rs`, `src/constraint_system.rs`, `src/failure_axis.rs` | `tests/adversarial_validation.rs`, guardian and constraint tests | Pass (code + tests) |
| FAC-008/014 | Facility and interface limits represented as explicit constraints | `src/constraint_system.rs`, `src/capacity_available_to_sced.rs` | constraint-system tests and deterministic SCED chain validation path | Pass (code + tests) |
| CIP-007 | System integrity and deterministic enforcement | `src/lib.rs`, `src/compliance.rs`, crate-level `#![deny(unsafe_code)]` modules | source policy plus CI lint/check workflow in `.github/workflows/rust-ci.yml` | Pass (policy + workflow) |
| CIP-010 | Configuration integrity and immutable-style traceability | `src/sovereign_trace.rs`, `src/sprint1.rs`, `src/sced_offer_chain.rs` | hash-chain functions and verifier report outputs (`src/bin/verifier.rs`, `src/bin/sced_chain.rs`) | Pass (code + verifier) |
| CIP-013 | Trust boundaries and supplier/input integrity framing | `src/universal_frontend.rs`, `src/crypto_pipeline.rs`, `src/interface_discovery.rs` | deterministic constraints and documented trust-boundary framing in specs | Pass (framing + code scaffold) |

## 4) Non-Conformance Register (Open Items)

| ID | Non-Conformance | Impact | Mitigation Plan | Owner | Target Date | Status |
|---|---|---|---|---|---|---|
| NCR-001 | Fresh local `cargo check` evidence currently blocked by host rustup/toolchain state on this workstation | Cannot claim new same-day local compile evidence from this host | Re-run checks in clean Ubuntu WSL session and store logs under `evidence/ci/` | Build/Release Owner | 2026-04-10 | Open |
| NCR-002 | Dependency vulnerability scan artifact not yet attached | Security review package incomplete | Run `cargo audit` (or equivalent) and archive output in `evidence/security-integrity/` | Security Owner | 2026-04-10 | Open |
| NCR-003 | Pre-submission sign-off bundle not assembled | Submission gate cannot be closed | Assemble evidence tree and reviewer checklist for end-to-end review | Program Owner | 2026-04-14 | Open |

## 5) Regulatory Assumptions (Versioned)

Assumptions version: `2026-04-07.v1`

1. This package is an assurance and deterministic evidence layer, not an autonomous market dispatch replacement.
2. L7 emergency actions are represented as explicit constrained outcomes and require external/operator authority.
3. Current compliance package scope is pilot evaluation and reproducible verification evidence.
4. Any transition to assisted-control authority requires a separate governance and validation gate.
5. Control-family mappings are obligation framing aids and must be validated by formal compliance/legal review before submission.

## 6) Sprint Priority Enforcement (April 2026)

| Standard | Priority Control | Deterministic Enforcement Path | Violation Action |
|---|---|---|---|
| BAL-001-2 | ACE and frequency response with constant B-factor | `src/reliability_controls.rs` (`compute_ace`, `evaluate_bal001`, `validate_constant_b_factor`) | `ErrBal001` |
| PRC-001 / PRC-024 | UFRT and ride-through envelope | `src/reliability_controls.rs` (`prc001_ufrt_trip_required`, `prc024_enforce_envelope`) | `Err003` / `Err004` |
| FAC-008-3 | Thermal SOL normal/emergency enforcement | `src/reliability_controls.rs` (`evaluate_fac008`) | `ErrFac008` |
| CIP-007-6 / CIP-011-2 | Default-deny ports/services and topology integrity | `src/reliability_controls.rs` (`enforce_cip007`, `enforce_cip011_topology_integrity`) | `HaltCip001` |

## Build Environment Guidance

Canonical build environment (submission/repro):
- Linux host (RHEL/Ubuntu) with project on native Linux filesystem.
- Required: Rust toolchain per `rust-toolchain.toml`, `cargo build --release`, and SHA-256 hash capture.

Developer convenience environment:
- WSL2 on Windows is acceptable, but builds are slower on `/mnt/c/...`.
- Prefer copying repo to `~/M.V.R.ESPRINT1` inside WSL for faster I/O.

Repro evidence expectations:
- Provide release build logs, binary hashes, and a small smoke test output.
- Example commands: `cargo build --release`, `Get-FileHash -Algorithm SHA256 target/release/*.exe`.

Appendix: Build Environment Guidance -> docs/BUILD_ENVIRONMENT_GUIDANCE.md
