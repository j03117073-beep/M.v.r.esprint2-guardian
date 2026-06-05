# Phase III Enforcement CLI Documentation

## Overview

The Phase III enforcement tool is a Python-based utility that enforces claim traceability, evidence linkage, and invariant alignment. It operates in a safe-first, read-only manner by default and requires explicit approval for any repository modifications.

## Installation

No installation required. The tool uses standard library Python3 and regex.

```bash
python3 tools/phase3_enforce/enforce.py --help
```

## Usage

### Dry-run (default, safe)

Perform a full enforcement scan and generate a report without modifying any repository files except generated audit outputs:

```bash
python3 tools/phase3_enforce/enforce.py --dry-run
```

**Output:**
- `phase_iii/ENFORCEMENT_REPORT.md` — detailed enforcement findings
- `phase_iii/enforcement_tx.log` — transaction log with scan metadata
- `phase_iii/enforcement_patch.diff` — preview of proposed changes (non-destructive)

### Apply changes (requires explicit approval)

Apply non-destructive updates and record a patch preview:

```bash
python3 tools/phase3_enforce/enforce.py --apply
```

**Output:**
- `phase_iii/ENFORCEMENT_REPORT.md` — updated enforcement report
- `phase_iii/enforcement_tx.log` — updated transaction log
- `phase_iii/enforcement_patch.diff` — generated patch diff for review

### Patch generation

The enforcement tool generates `phase_iii/enforcement_patch.diff` in both dry-run and apply modes. It contains any diff between the previous and current enforcement artifacts.

**Preconditions:**
- Must first review `phase_iii/enforcement_patch.diff` from a `--dry-run`.
- Changes are not committed automatically; operator must commit from the repository root.

### Validate a single claim

Re-run enforcement validation (CV ruleset) for a specific claim:

```bash
python3 tools/phase3_enforce/enforce.py --revalidate "demo scenario playback"
```

## Enforcement Rules

### Claim Validation (CV Ruleset)

A claim is `VALIDATED` only if ALL of the following are true:

1. **CV-1: Structural Validity**
   - Claim exists in `phase_ii/compliance/CLAIM_TRACEABILITY_MATRIX.md`.
   - Claim has a non-empty "Source Location" or explicit note if unsupported.

2. **CV-2: Evidence Sufficiency**
   - At least one evidence artifact is linked to the claim (via explicit linkage in `EVIDENCE_INDEX.md` or fuzzy keyword matching).
   - Evidence is classified as reproducible OR a reproducible regeneration path exists.

3. **CV-3: Invariant Alignment**
   - No conflicting invariant state detected.
   - If an invariant applies to the claim, its status must be one of: `VERIFIED`, `UNRESOLVED`, `DEFERRED`.

4. **CV-4: Exit Compatibility**
   - Claim does not violate `phase_iii/EXIT_CRITERIA.md`.

### Claim States

| State | Meaning | Action |
|---|---|---|
| `VALIDATED` | Meets CV rules and is audit-ready | No action required |
| `PENDING_EVIDENCE` | Has source but lacks evidence linkage | Add evidence or update linkage |
| `FAILED_VALIDATION` | Meets some but not all CV rules | Review and fix violations |
| `INVALID_STRUCTURE` | No source location, unsupported | Implement, document as unsupported, or remove |

### Evidence States

| State | Meaning | Action |
|---|---|---|
| `LINKED_VALID` | Linked to claim and reproducible | No action required |
| `ORPHANED` | Not linked to any claim | Link to claim or remove |
| `INVALIDATED` | Link broken or evidence missing | Repair link or regenerate evidence |

### Invariant States

| State | Meaning | Action |
|---|---|---|
| `VERIFIED` | Proven by evidence or tests | No action required |
| `UNRESOLVED` | Known but not yet proven | Document why and create task |
| `VIOLATED` | Evidence contradicts invariant | High priority; investigate |
| `DEFERRED` | Intentionally deferred; reason documented | Review deferral reason; set deadline |

## Enforcement Engine Cycle

When you run `--dry-run` or `--apply`, the tool performs:

1. **INGEST** — Parse `phase_ii/`, `phase_iii/` for claims, evidence, invariants
2. **MAP** — Build indices linking claims ↔ evidence ↔ invariants
3. **VALIDATE** — Apply CV ruleset to classify each claim
4. **RESOLVE** — Attempt safe automated actions (evidence linking, artifact mirroring)
5. **REPORT** — Generate markdown summaries and transaction log

## Output Files

### phase_iii/ENFORCEMENT_REPORT.md
Primary enforcement output. Includes:
- Claim count and linkage summary
- Detailed claim-to-evidence mappings
- Reproducibility audit
- Unlinked claims and orphaned evidence

### phase_iii/enforcement_tx.log
Immutable transaction log. Records:
- Scan timestamp
- Tool version
- Operator (git user if available)
- Claims scanned, evidence scanned, invariants classified
- Any automated changes proposed

### phase_iii/enforcement_patch.diff
Proposed changes in unified diff format (for review before `--apply`).

## Integration with CI

### GitHub Actions Example

```yaml
name: Phase III Enforcement

on: [pull_request]

jobs:
  enforce:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-python@v4
        with:
          python-version: '3.10'
      - name: Run Phase III Enforcement (dry-run)
        run: python3 tools/phase3_enforce/enforce.py --dry-run
      - name: Check for regressions
        run: |
          if grep -q 'INVALID_STRUCTURE' phase_iii/ENFORCEMENT_REPORT.md; then
            echo "Regression: new INVALID_STRUCTURE claims detected"
            exit 1
          fi
```

This ensures PRs cannot introduce new unsupported claims.

## Troubleshooting

### "No evidence artifacts scanned"

- Verify `phase_ii/compliance/EVIDENCE_INDEX.md` exists and has a properly formatted table.
- Run a `--dry-run` and check `phase_iii/ENFORCEMENT_REPORT.md` for parse errors.

### "Claims missing or claim count low"

- Verify `phase_ii/compliance/CLAIM_TRACEABILITY_MATRIX.md` has a properly formatted table.
- Check that backticks around claim names are present and correctly formatted.

### "Linked claims not matching"

- Verify the "Linked Claims" column in `EVIDENCE_INDEX.md` uses backtick notation (e.g., `` `claim-name` ``).
- Run `enforce.py` with debug output: `python3 -u tools/phase3_enforce/enforce.py --dry-run 2>&1 | grep -i linked`.

## Future Enhancements

- [ ] Manifest-based enforcement (`.phase3-enforce.json`)
- [ ] Automated evidence regeneration (re-run verifier binaries)
- [ ] Git-backed transaction log with immutability verification
- [ ] Enforcement rules customization via config file
- [ ] Web UI for audit reports
