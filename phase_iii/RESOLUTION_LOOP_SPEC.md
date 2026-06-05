# Phase III Resolution Loop Specification (Design)

## Purpose
Define the automated "Resolution Loop" that transitions the Phase III enforcement engine from detection-only to a closed-loop system that can (1) ingest artifacts, (2) map and validate relationships, (3) attempt automated resolution actions where safe, and (4) reclassify & report state changes.

## Objectives
- Close the enforcement loop by enabling reproducible evidence mirroring and re-validation.
- Provide precise data models and rules the enforcement engine consumes and emits.
- Specify safe automated resolution actions and strict failure-handling boundaries.
- Define CI integration points and operator CLI for manual intervention.

## High-level Cycle
STEP 1 — INGEST
- Scan `phase_ii/` and `phase_iii/` for claim, evidence, and invariant artifacts.
- Accept formats: Markdown tables, CSV, and a small enforcement JSON/YAML schema (`.phase3-enforce.json`).

STEP 2 — MAP
- Build in-memory indices:
  - claims → evidence list
  - claims → invariants list
  - evidence → provenance metadata
- Use `CLAIM_TRACEABILITY_MATRIX.md`, `EVIDENCE_INDEX.md`, and `OPEN_INVARIANTS.md` as canonical inputs.

STEP 3 — VALIDATE
- Apply CV ruleset (CV-1..CV-4) to each claim.
- Validate invariants have one of the allowed states: `VERIFIED|VIOLATED|UNRESOLVED|DEFERRED`.
- Classify evidence: `LINKED_VALID|ORPHANED|INVALIDATED`.

STEP 4 — RESOLVE (Automated, safe-first)
- For ORPHANED runtime `/tmp/` artifacts: attempt to locate repo-local mirror at `phase_ii/evidence/`; if absent, copy matching artifacts into `phase_ii/evidence/` only when run by a privileged operator (explicit CLI flag `--mirror` required).
- For PENDING_EVIDENCE claims where a repo-local verifier binary exists (e.g., `pilot_demo`, `verifier`, `sced_chain`): invoke the verifier in a sandboxed process to re-generate evidence and capture logs into `phase_ii/evidence/`.
- For INVALID_STRUCTURE (missing source): mark claim `INVALID_STRUCTURE` and suggest remediation; do NOT auto-create source.
- For UNRESOLVED invariants: create a prefixed draft in `phase_iii/invariant_actions/INV_<id>.md` containing suggested next steps and a reproducible command template; mark invariant `DEFERRED` only with a reason.
- All automated changes must be recorded in an enforcement transaction log `phase_iii/enforcement_tx.log` and produce a git-friendly patch file `phase_iii/enforcement_patch.diff` for manual review before committing.

STEP 5 — REPORT
- Update `CLAIM_TRACEABILITY_MATRIX.md`, `EVIDENCE_INDEX.md`, and `OPEN_INVARIANTS.md` with new classifications, but do not auto-commit to `main` unless operator confirms.
- Emit `phase_iii/ENFORCEMENT_REPORT.md` summarizing diffs and actions performed.

## Data Model (Minimal)
- enforcement manifest (`.phase3-enforce.json`)
  - claims: [{id, title, source_paths[], invariants[]}] 
  - evidence: [{id, path, type, reproducible:boolean, linked_claims[]}] 
  - invariants: [{id, description, status, linked_claims[], next_action}] 

- CLI uses this manifest as the authoritative transient state during a run.

## Enforcement Rules (Safe by default)
- No destructive actions without `--apply` (default is `--dry-run`).
- Mirroring and sanitizer actions require `--mirror` and `--apply` together.
- Any automated change that touches tracked files creates `phase_iii/enforcement_patch.diff` and `phase_iii/enforcement_tx.log`.
- When a claim transitions to `FAILED_VALIDATION`, the system will not auto-delete the claim; it will set state and open an invariant action.

## Failure Handling
- On detection of invariant contradictions: halt dependent claim resolution, mark claims `PENDING_EVIDENCE` or `FAILED_VALIDATION` as appropriate, and create a blocking task file under `phase_iii/tasks/`.
- All unresolvable issues remain `UNRESOLVED` and require manual resolution.

## CLI & Usage
- Proposed CLI: `cargo run --bin phase3_enforce -- scan --manifest .phase3-enforce.json [--dry-run] [--mirror] [--apply]`
- Minimal commands:
  - `scan` — perform full loop in dry-run and write `ENFORCEMENT_REPORT.md` and `enforcement_tx.log`.
  - `apply` — apply non-destructive changes (mirror + update MD files) after operator review.
  - `revalidate <claim-id>` — re-run CV ruleset for a single claim.

## CI Integration
- Add a workflow `phase3-enforce.yml` that runs the enforcement engine in `dry-run` on PRs touching `phase_ii/` or `phase_iii/`.
- Gate merges on no new `INVALID_STRUCTURE` states introduced by the PR.

## Implementation Roadmap (First Batch)
1. Create `phase_ii/evidence/` and a small script `scripts/mirror_evidence.sh` to copy `/tmp/...` artifacts into repo-local evidence (manual operator approval needed).
2. Implement a small Rust (or Python) enforcement tool `phase3_enforce` that implements ingest→map→validate and outputs `ENFORCEMENT_REPORT.md` (dry-run first).
3. Add enforcement tx logging and patch generation (`enforcement_patch.diff`).
4. Run enforcement tool, review `enforcement_patch.diff`, then `--apply` to update MD files and commit from operator machine.
5. Add CI `phase3-enforce` dry-run step.

## Security & Audit Considerations
- Mirroring external runtime logs must be done deliberately to avoid injecting untrusted files; require operator review.
- Enforcement logs must be immutable after generation and stored with provenance (git commit hash, timestamp, operator id).

## Next Actions for Me
- I will create `scripts/mirror_evidence.sh` and a minimal enforcement tool skeleton in `tools/phase3_enforce/` (optional language: Rust).  Confirm and I will implement the first-batch artifacts.
