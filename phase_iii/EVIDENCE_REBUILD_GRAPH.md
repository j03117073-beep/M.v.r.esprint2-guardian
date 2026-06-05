# Phase III Evidence Rebuild Graph

## Deterministic Reconstruction DAG

This document defines the strict evidence dependency graph for MVRE-SPRINT1.
No downstream artifact may be generated without its upstream parents being fully validated and cryptographically locked.

```text
       [Global Entropy Seed] (0xDEADBEEF42D070C1)
                       │
                       ▼
          [Static Topology Matrices] (D)
                       │
                       ▼
       (C) cargo run --bin determinism-check
                       │
                       ▼
          [Executable Artifacts]
                       │
                       ▼
 phase_ii/artifacts/logs/determinism_run.log
                       │
                       ▼
       (C) python3 tools/generate_solver_report.py
                       │
                       ▼
 phase_ii/artifacts/determinism_report.json
                       │
                       ▼
=========================================
[CLOSURE STATE: 100% SECURE & REPRODUCIBLE]
=========================================
```

## 1. Rebuild Pipeline Stages

### Stage 1: Core Kernel Simulation (Determinism Tracing)
- **Upstream (Seed/Inputs):**
  - `phase_ii/determinism/topology_matrix.json`
  - `phase_ii/determinism/load_profiles.json`
- **Rebuild Trigger Command:**
  ```bash
  cargo run --bin determinism-check --release -- --input phase_ii/determinism/topology_matrix.json --profiles phase_ii/determinism/load_profiles.json --output phase_ii/artifacts/logs/determinism_run.log
  ```
- **Downstream Lock:** `phase_ii/artifacts/logs/determinism_run.log`

### Stage 2: Synthesis & Reporting (Boundary Alignment)
- **Upstream (Stage 1 Output):**
  - `phase_ii/artifacts/logs/determinism_run.log`
- **Rebuild Trigger Command:**
  ```bash
  python3 tools/generate_solver_report.py --log phase_ii/artifacts/logs/determinism_run.log --output phase_ii/artifacts/determinism_report.json
  ```
- **Downstream Lock:** `phase_ii/artifacts/determinism_report.json`

### Stage 3: Formal Verification (Proof Verification Engine)
- **Upstream (Formal Theorems):**
  - `verification/theorems/no_deadlock.v`
- **Rebuild Trigger Command:**
  ```bash
  coqc -Q verification/theorems/ Kernel verification/theorems/no_deadlock.v > phase_ii/artifacts/logs/coq_proof_output.log
  ```
- **Downstream Lock:** `phase_ii/artifacts/logs/coq_proof_output.log`

## 2. Integrity Check Rules

1. **Strict Path Isolation:** No generation command may reference files outside repository boundaries.
2. **No Ambient Storage Access:** Accessing network interfaces, ephemeral system directories, or user home directories during execution is forbidden.
3. **Hash Verification:** Post-rebuild, generated files must be parsed and their hashes must match the entries defined inside `phase_iii/EVIDENCE_INDEX.md`.
