Phase III Enforcement Tool (prototype)

This folder contains a minimal enforcement prototype `enforce.py` that performs a dry-run enforcement pass.

Usage

```bash
python3 tools/phase3_enforce/enforce.py --dry-run
python3 tools/phase3_enforce/enforce.py --apply
```

Files

- `enforce.py` — minimal prototype: parses `phase_ii/compliance/CLAIM_TRACEABILITY_MATRIX.md` and `phase_ii/compliance/EVIDENCE_INDEX.md`, writes `phase_iii/ENFORCEMENT_REPORT.md`, `phase_iii/enforcement_tx.log`, and `phase_iii/enforcement_patch.diff`.

Notes

- This is intentionally conservative: default is dry-run and it does not modify repository files.
- Next steps: implement robust markdown parsing, manifest ingestion, and safe `--apply` behavior.
