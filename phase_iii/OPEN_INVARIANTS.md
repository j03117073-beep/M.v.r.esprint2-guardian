# Phase III Open Invariants Register

## Purpose
Document everything that is not yet fully proven or verified in a way that an independent reviewer can immediately see remaining risk.

### Open Invariant 1: Missing CLI Claims
- Description: `scenario_runner` and `ise_runner` are documented claims with no implementation in this checkout.
- Status: UNRESOLVED
- Impact: The repository claims more runtime paths than are currently present.
- Next Action: confirm whether these binaries are intentionally unsupported or add a documented disclaimer to `README.md`.

### Open Invariant 2: Phase III Prediction Payload Coverage
- Description: `README.md` and `OPERATIONAL_MANUAL.md` claim Phase III prediction payload sampling and validation, but the evidence path is limited.
- Status: UNRESOLVED
- Impact: A reviewer cannot fully verify the prediction payload workflow from current artifacts alone.
- Next Action: inventory the exact `sced_chain` sample/validate artifacts and add traceability entries.

### Open Invariant 3: Independent Audit Readiness
- Description: The current repository lacked an explicit Phase III audit framework until `phase_iii/` was created.
- Status: UNRESOLVED
- Impact: Audit readiness depends on documentation and reviewer instructions.
- Next Action: validate that `phase_iii/` artifacts are complete and linked from top-level docs.

### Open Invariant 4: Evidence Artifact Availability
- Description: Some evidence artifacts are referenced as `/tmp/...` which may not be preserved across environments.
- Status: UNRESOLVED
- Impact: An auditor may not find reproducible artifacts without a repository-local evidence directory.
- Next Action: move or mirror important evidence artifacts into `phase_ii/` or `phase_iii/` and update references.

### Open Invariant 5: Phase III Exit Criteria Clarity
- Description: There is no explicit exit checklist for completing Phase III in the repository today.
- Status: UNRESOLVED
- Impact: Team and reviewer may disagree on when Phase III is done.
- Next Action: complete `phase_iii/EXIT_CRITERIA.md` and align it with this framework.

### Open Invariant 6: External Reviewer Independence
- Description: The repo currently assumes author knowledge in many places (e.g. ad hoc artifact paths, undocumented commands).
- Status: UNRESOLVED
- Impact: A stranger may still need to contact the team to understand how to reproduce or audit results.
- Next Action: add a short `phase_iii/README` or top-level `README` section showing the exact review path.
