# Phase III Invariant Resolution Matrix

## Purpose

Create a concise audit-facing summary that maps the highest-risk unresolved invariants to their current state, evidence status, and the concrete closure actions needed to reach Phase III completion.

## Matrix

| Invariant | Status | Evidence | Closure Action |
|---|---|---|---|
| INV-003: Evidence Artifact Availability | Open | `/tmp/phase_ii_repro/logs/*` artifacts are referenced in `EVIDENCE_INDEX.md` and `CLAIM_TRACEABILITY_MATRIX.md` | Mirror or reproduce the referenced runtime artifacts into a repo-local path such as `phase_ii/evidence/`, update `EVIDENCE_INDEX.md`, and remove or clearly document any remaining `/tmp/` references as orphaned with regeneration instructions. |
| INV-006: External Reviewer Independence | Open | `phase_iii/REVIEWER_START.md` exists, but audit independence depends on the completeness of reviewer instructions and path validation | Validate the reviewer path on a clean checkout; add missing explicit commands, artifact expectations, and known environment requirements; ensure the guide covers every claim and every evidence artifact without author contact. |
| INV-007: Phase III Exit Criteria Alignment | Open | `phase_iii/EXIT_CRITERIA.md` defines completion criteria, but the repo does not currently show a completed checklist or reviewer acceptance evidence | Create a Phase III completion checklist or companion `phase_iii/COMPLETION_CHECKLIST.md`; annotate each criterion with pass/fail status and link to the supporting docs or evidence. |

## Observations

- The highest-value gap is reproducible evidence. The audit framework already calls this out explicitly in `phase_iii/EVIDENCE_INDEX.md`, `phase_iii/TASKS.md`, and `phase_iii/INDEX.md`.
- The reviewer independence gap is lower-risk only if the reviewer guide is fully executable. Right now the repo provides a guide, but it has not been validated from first-principles on a clean checkout.
- Exit criteria are defined, but the current repo state still needs an explicit “this is complete / this is deferred” checklist to satisfy external audit expectations.

## Recommended next steps

1. Resolve INV-003 by mirroring `/tmp/` evidence into `phase_ii/evidence/` or by adding explicit regeneration commands and marking orphaned artifacts clearly.
2. Validate `phase_iii/REVIEWER_START.md` against a clean clone, then update it to remove any assumptions about author knowledge.
3. Add a concrete completion checklist that ties `phase_iii/EXIT_CRITERIA.md` to observable repo state.

## Notes

- This matrix is intended to be the first artifact a reviewer or maintainer checks when deciding whether Phase III is audit-ready.
- After these items are addressed, the next highest-value work is to confirm the validation/build suite and capture the results as audit evidence.
