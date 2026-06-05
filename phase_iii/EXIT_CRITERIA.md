# Phase III Exit Criteria

## Purpose
Define the objective conditions that must be true before Phase III is complete.

## Completion Criteria
The repository reaches Phase III completion when all of the following are true:

1. `phase_iii/PHASE_III_CHARTER.md` is present and describes the Phase III mission.
2. `phase_ii/compliance/CLAIM_TRACEABILITY_MATRIX.md` enumerates every substantive claim and links it to source, verification, evidence, and status.
3. `phase_ii/compliance/EVIDENCE_INDEX.md` references every known Phase I and Phase II evidence artifact used for audit validation.
4. `phase_iii/OPEN_INVARIANTS.md` captures all unresolved claims, missing paths, and uncertainties.
5. `phase_iii/AUDIT_CHECKLIST.md` provides a repeatable, reviewer-facing audit path.
6. `phase_iii/EXIT_CRITERIA.md` clearly defines completion in reviewer terms.
7. All `VERIFIED` claims have evidence artifacts that are accessible from the repository or by documented reproducible steps.
8. All `OPEN` and `PARTIAL` items include a next action that is visible to a reviewer.
9. The repository includes no hidden audit dependencies requiring direct author guidance.
10. A reviewer can answer the audit question without talking to the team:
    - what is claimed,
    - what was tested,
    - what passed,
    - what remains open.

## Reviewer Acceptance
For a reviewer to accept Phase III completion, they must confirm:

- the claim matrix is complete and honest,
- the evidence index is materially accurate,
- open invariants are transparent,
- the audit checklist can be executed without missing steps,
- the phase is auditable from the repository contents alone.

## Notes
- Phase III is complete only when the repository is in a state where an independent audit can start immediately.
- If new claims or major design changes are introduced after Phase III, the framework must be updated and Phase III must be revalidated.
