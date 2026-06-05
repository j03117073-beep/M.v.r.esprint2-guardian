# Phase III Audit Checklist

## Objective
Provide a step-by-step list for an outside reviewer to validate the repository’s Phase III readiness.

## 1. Environment and Repository Setup
- [ ] Clone the repository at the root commit for Phase III review.
- [ ] Confirm `cargo test --all` passes on the expected Rust toolchain.
- [ ] Confirm the repository layout includes `phase_ii/`, `phase_iii/`, `src/`, `tests/`, and top-level docs.
- [ ] Identify relevant evidence directories: `phase_ii/determinism/`, `phase_ii/adversarial/`, `phase_ii/reproduction_logs/`.

## 2. Review Phase III Framework
- [ ] Read `phase_iii/PHASE_III_CHARTER.md`.
- [ ] Read `phase_iii/OPEN_INVARIANTS.md`.
- [ ] Read `phase_iii/EXIT_CRITERIA.md`.
- [ ] Read `phase_ii/compliance/EVIDENCE_INDEX.md`.
- [ ] Read `phase_ii/compliance/CLAIM_TRACEABILITY_MATRIX.md`.

## 3. Validate Claims
- [ ] Confirm every claim in `README.md`, `OPERATIONAL_MANUAL.md`, `MERGE_PLAN.md`, and branch analysis docs is captured in `CLAIM_TRACEABILITY_MATRIX.md` or documented as unsupported.
- [ ] Verify each claim has a source code reference or an explicit note if no implementation exists.
- [ ] Verify each claim has at least one verification method or a stated reason why it is pending.
- [ ] Verify each claim points to evidence artifacts when available.

## 4. Verify Evidence Artifacts
- [ ] Confirm the evidence files listed in `EVIDENCE_INDEX.md` exist at the referenced paths.
- [ ] Confirm artifacts are consistent with the claimed verification method.
- [ ] Confirm evidence categories cover Phase I and Phase II delivery.
- [ ] Confirm missing evidence is explicitly called out.

## 5. Review Open Invariants
- [ ] Confirm `OPEN_INVARIANTS.md` contains all unresolved claims, unsupported binaries, and pending integration items.
- [ ] Confirm each open invariant includes a description, current status, and next action.
- [ ] Confirm no open invariant is hidden or implicitly omitted.

## 6. Confirm Exit Criteria
- [ ] Confirm `EXIT_CRITERIA.md` defines what must be true before closing Phase III.
- [ ] Confirm success is defined in reviewer-facing terms: the project is auditable.
- [ ] Confirm there is a clear handoff from audit framework to any remaining repo work.

## 7. Audit Report
- [ ] Prepare a short review summary describing:
  - which claims are verified,
  - which evidence artifacts are present,
  - which items remain open,
  - whether the repo is auditable without external guidance.
- [ ] Record any gaps in traceability, evidence, or documentation.
- [ ] Identify any follow-up action items necessary to complete Phase III.
