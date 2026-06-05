# Phase III Tasks Generated from OPEN_INVARIANTS

## INV-001: Missing CLI Claims (scenario_runner, ise_runner)

**Status**: UNRESOLVED  
**Priority**: MEDIUM  
**Owner**: TBD  

### Description
The repository documents claims for `scenario_runner` and `ise_runner` CLI tools that are not present in the current checkout.

### Actions
- [ ] Confirm whether these binaries are intentionally unsupported.
- [ ] If unsupported: add a documented disclaimer to `README.md` and mark claims as `INVALID_STRUCTURE`.
- [ ] If supported: implement the binaries or provide reproduction instructions from external sources.

### Acceptance Criteria
- [ ] Claim state for both is no longer `INVALID_STRUCTURE`.
- [ ] `README.md` explicitly documents what runtime paths are available and what are not.

---

## INV-002: Phase III Prediction Payload Coverage

**Status**: UNRESOLVED  
**Priority**: MEDIUM  
**Owner**: TBD

### Description
The `README.md` and `OPERATIONAL_MANUAL.md` claim Phase III prediction payload sampling and validation, but evidence linkage is incomplete.

### Actions
- [ ] Inventory the exact `sced_chain` sample/validate outputs.
- [ ] Create or locate sample payloads in `phase_ii/` or `phase_iii/`.
- [ ] Add explicit evidence entries to `phase_ii/compliance/EVIDENCE_INDEX.md`.
- [ ] Update `phase_ii/compliance/CLAIM_TRACEABILITY_MATRIX.md` to link the claim.

### Acceptance Criteria
- [ ] Claim `Phase III prediction payload sampling and validation` transitions from `PENDING_EVIDENCE` to `VALIDATED`.

---

## INV-003: Evidence Artifact Availability (Non-reproducible /tmp/ artifacts)

**Status**: UNRESOLVED  
**Priority**: HIGH  
**Owner**: TBD

### Description
Runtime evidence artifacts are stored under `/tmp/...` which are transient and non-reproducible. An auditor will not find these unless they're mirrored into the repository.

### Actions
- [ ] Run `bash scripts/mirror_evidence.sh --source /tmp/phase_ii_repro/logs --dest phase_ii/evidence --confirm` to copy artifacts (when available).
- [ ] Alternatively, document a reproducible path to regenerate these artifacts or confirm they're not needed.
- [ ] Update `phase_ii/compliance/EVIDENCE_INDEX.md` to mark `/tmp/` artifacts as `ORPHANED` or update reproducibility status.

### Acceptance Criteria
- [ ] No `/tmp/` artifacts remain in `EVIDENCE_INDEX.md`, OR all are marked as reproducible with step-by-step instructions.
- [ ] `phase_ii/evidence/` contains all mirrored runtime logs.

---

## INV-004: SCED Offer-Chain Verification Evidence

**Status**: UNRESOLVED  
**Priority**: MEDIUM  
**Owner**: TBD

### Description
The claim `SCED offer-chain verification` references test vectors and scripts but has no explicit evidence linkage.

### Actions
- [ ] Verify that `test_vectors/` and `scripts/` contain the expected SCED verification artifacts.
- [ ] Document the exact run command and expected output.
- [ ] Add evidence entries to `phase_ii/compliance/EVIDENCE_INDEX.md` (e.g., sample run logs, test vector metadata).
- [ ] Link the claim in `phase_ii/compliance/CLAIM_TRACEABILITY_MATRIX.md`.

### Acceptance Criteria
- [ ] Claim state transitions from `PENDING_EVIDENCE` to `VALIDATED`.

---

## INV-005: Dashboard Server Evidence

**Status**: UNRESOLVED  
**Priority**: LOW  
**Owner**: TBD

### Description
The `dashboard` server claim points to `/tmp/phase_ii_repro/logs/dashboard_run.log` which is non-reproducible.

### Actions
- [ ] Either mirror the log into `phase_ii/evidence/` or document reproducible startup steps.
- [ ] Add a test or integration script that validates dashboard startup.
- [ ] Link updated evidence in `EVIDENCE_INDEX.md`.

### Acceptance Criteria
- [ ] Claim `dashboard server` has reproducible evidence or a clear reproduction path.

---

## INV-006: External Reviewer Independence

**Status**: UNRESOLVED  
**Priority**: HIGH  
**Owner**: TBD

### Description
The repository currently assumes author knowledge in many places. An external reviewer needs a clear, standalone audit path.

### Actions
- [ ] Create `phase_iii/REVIEWER_START.md` with:
  - Exact checkout instructions.
  - Step-by-step audit commands.
  - Expected outputs and pass/fail criteria.
  - Links to all framework documents.
- [ ] Validate the reviewer path on a clean checkout.

### Acceptance Criteria
- [ ] A reviewer can start from `REVIEWER_START.md` and complete the audit independently.

---

## INV-007: Phase III Exit Criteria Alignment

**Status**: UNRESOLVED  
**Priority**: HIGH  
**Owner**: TBD

### Description
Exit criteria exist but may not be fully satisfied by current state.

### Actions
- [ ] Review `phase_iii/EXIT_CRITERIA.md` against current `phase_iii/` contents.
- [ ] For each criterion, validate and document the status.
- [ ] Create a completion checklist in `phase_iii/COMPLETION_CHECKLIST.md`.

### Acceptance Criteria
- [ ] `phase_iii/COMPLETION_CHECKLIST.md` shows all criteria met or explicitly deferred.

