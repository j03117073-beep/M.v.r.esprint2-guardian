# Phase III Independent Reviewer Start Guide

## Welcome

This guide helps an external (independent) reviewer audit the M.V.R.ESPRINT1 system without author contact. Start here.

## What is Phase III?

Phase III transitions the project from:

> "We can reproduce the results"

to:

> "An independent reviewer can audit the results"

This means every claim, every piece of evidence, and every unresolved item is documented and traceable in the repository.

## Quick Start (15 minutes)

### 1. Environment Setup

```bash
# Clone the repository at Phase III commit
git clone <repo-url>
cd M.v.r.esprint1-g

# Verify Rust toolchain
cargo --version
# Expected: cargo X.Y.Z or later

# Run a basic build check
cargo check
```

### 2. Locate Phase III Documentation

All Phase III artifacts are in the `phase_iii/` directory:

```bash
ls -la phase_iii/
```

**Key files:**
- `PHASE_III_CHARTER.md` — What Phase III is and why
- `CLAIM_TRACEABILITY_MATRIX.md` — Every claim + evidence + status
- `EVIDENCE_INDEX.md` — Master list of all evidence artifacts
- `OPEN_INVARIANTS.md` — What is NOT yet proven
- `AUDIT_CHECKLIST.md` — Step-by-step audit instructions
- `EXIT_CRITERIA.md` — What "Phase III complete" means
- `ENFORCEMENT_REPORT.md` — Latest enforcement scan results

### 3. Read the Charter and Matrix

```bash
# Understand the Phase III mission
cat phase_iii/PHASE_III_CHARTER.md

# See every claim and its status
cat phase_ii/compliance/CLAIM_TRACEABILITY_MATRIX.md
```

Key things to look for:
- How many claims are `VALIDATED` vs `PENDING_EVIDENCE`?
- Are there any `INVALID_STRUCTURE` claims (red flag)?
- Do sources exist for all claims?

### 4. Audit the Evidence

```bash
# See all evidence artifacts and their reproducibility
cat phase_ii/compliance/EVIDENCE_INDEX.md

# Verify repo-local evidence exists
ls phase_ii/evidence/
ls phase_ii/determinism/
ls phase_ii/adversarial/
```

**Questions to ask:**
- Are all reproducible artifacts present?
- Can I regenerate non-reproducible evidence using the documented steps?
- Are there any orphaned or missing artifacts?

### 5. Identify Open Items

```bash
# See what is unresolved
cat phase_iii/OPEN_INVARIANTS.md
cat phase_iii/TASKS.md
```

**This is normal.** Not everything is proven yet. The audit question is: *are the open items transparent and tracked?*

### 6. Run the Enforcement Tool

```bash
# Generate a fresh enforcement report
python3 tools/phase3_enforce/enforce.py --dry-run

# Review the output
cat phase_iii/ENFORCEMENT_REPORT.md
```

**Interpretation:**
- `VALIDATED` claims: evidence is present and linked, auditable.
- `PENDING_EVIDENCE` claims: source exists but evidence needs review or regeneration.
- `INVALID_STRUCTURE` claims: unsupported or placeholder claims; should be rare.

## Full Audit (60+ minutes)

Follow the structured checklist:

```bash
cat phase_iii/AUDIT_CHECKLIST.md
```

### Detailed Steps

#### Step 1: Verify Claims

For each claim in `CLAIM_TRACEABILITY_MATRIX.md`:

1. Verify the source code location exists:
   ```bash
   ls <source-location>  # e.g., src/bin/demo.rs
   ```

2. Understand the claim's purpose by reading the source.

3. Check the verification method:
   - Can you run the command specified?
   - Does it produce the expected output?

#### Step 2: Trace Evidence

For each evidence artifact in `EVIDENCE_INDEX.md`:

1. Verify the artifact exists:
   ```bash
   ls <location>  # e.g., phase_ii/determinism/pilot_attestation_log_run_1.json
   ```

2. Check "Linked Claims" — is it linked to at least one claim?

3. If reproducible, try to regenerate it:
   ```bash
   # Example: regenerate a demo log
   cargo run --bin demo -- normal > /tmp/demo_test.log
   ```

#### Step 3: Validate Invariants

Review `phase_ii/invariant_register.md` and `OPEN_INVARIANTS.md`:

1. For each `VERIFIED` invariant, check if evidence supports it.
2. For each `UNRESOLVED` invariant, note what would be needed to verify it.
3. Check that no invariants contradict each other.

#### Step 4: Test Reproducibility

Run the reproduction scripts:

```bash
# Run basic tests
cargo test --lib

# Run determinism checks (if available)
cargo test --test adversarial_validation

# Run a demo scenario
cargo run --bin demo -- normal

# Run the verifier
cargo run --bin pilot_demo
cargo run --bin verifier pilot_attestation_log.json
```

Expected outcome:
- All tests pass
- Outputs are deterministic
- No uncaught panics

## Audit Report Template

After completing the audit, fill in this template:

```markdown
# Phase III Audit Report
Date: [date]
Reviewer: [your name or "Anonymous"]

## Summary
- Total claims: [#]
- Validated: [#] ([%])
- Pending evidence: [#] ([%])
- Invalid structure: [#] ([%])

## Reproducibility Assessment
- [X] Repository builds without errors
- [X] Basic tests pass
- [X] Demo scenarios run
- [X] Determinism verifiable

## Evidence Coverage
- [X] All reproducible artifacts located
- [X] Linked evidence sufficient for validated claims
- [X] Orphaned evidence: [list if any]

## Unresolved Items
- [List any open invariants that should be closed]
- [List any missing evidence that would improve auditability]

## Recommendations
- [Any improvements for future phases]

## Verdict
Phase III audit: PASS / CONDITIONAL PASS / FAIL

Reason: ...
```

## Questions to Answer

After the audit, you should be able to answer:

1. **What claims does the system make?** — From `CLAIM_TRACEABILITY_MATRIX.md`.
2. **What was tested?** — From `EVIDENCE_INDEX.md` and reproducibility paths.
3. **What passed?** — From enforcement report and test results.
4. **What remains open?** — From `OPEN_INVARIANTS.md` and `TASKS.md`.
5. **Can I audit this independently?** — ANSWER: Yes, if all of the above are clear and traceable.

## Resources

- [Phase III Charter](PHASE_III_CHARTER.md)
- [Claim Traceability Matrix](CLAIM_TRACEABILITY_MATRIX.md)
- [Evidence Index](EVIDENCE_INDEX.md)
- [Open Invariants](OPEN_INVARIANTS.md)
- [Audit Checklist](AUDIT_CHECKLIST.md)
- [Exit Criteria](EXIT_CRITERIA.md)
- [Enforcement CLI Docs](ENFORCEMENT_CLI_DOCS.md)

## Support

This repository is designed to be auditable *without* author contact. If you:

- Find a missing file or broken link: This is a Phase III failure.
- Cannot reproduce a claimed result: Document which claim and why.
- Disagree with an evaluation: Your feedback improves the audit framework.

All findings should be recorded and shared with the team.

## Next Steps

1. Clone and set up the environment.
2. Read the Phase III charter and matrix.
3. Follow the audit checklist step-by-step.
4. Run the enforcement tool.
5. Fill in the audit report template.
6. Submit findings to the project team.
