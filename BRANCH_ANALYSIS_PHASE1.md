# Branch Integration Analysis - Phase 1: Discovery

**Generated**: 2026-05-29
**Status**: Pre-merge analysis (no changes yet)

---

## 1. Branch Inventory

### Active Branches

| Branch | Latest Commit | Author | Date | Total Commits |
|--------|---------------|--------|------|----------------|
| **main** | `04362ce` | mvre-sprint1 | 2026-05-21 14:13:59 | 19 |
| **origin/codex/deterministic-core-v1** | `7dcf3c4` | obienova | 2026-05-13 04:50:54 | 52 |
| **origin/verified-kernel** | `43f49bf` | mvre-sprint1 | 2026-05-22 21:29:25 | 61 |

### Branch Status

- **main**: Most recent update 2026-05-21
- **verified-kernel**: Newest branch, created after main with replay validation work
- **codex/deterministic-core-v1**: Experimental/development branch, oldest of the three feature branches

---

## 2. Commit Topology

### Relationship to Common Ancestor `d432abc` (2026-03-22 15:14:19)

```
Divergence Point: d432abc Install TPM system deps in CI
│
├─ main (3 commits ahead of codex/deterministic-core-v1)
│  └─ 04362ce Add files via upload
│     3ad7d0e fix: correct format string in adversarial_validation.rs CI BUILD GATE telemetry
│     70e782a rewrite project documentation for current verified state
│
├─ origin/codex/deterministic-core-v1 (36 commits ahead of main)
│  └─ a50992e..7dcf3c4 (extensive deterministic core development)
│
└─ origin/verified-kernel (45 commits ahead of main from common ancestor, 9 beyond codex/deterministic-core-v1)
   └─ 49f07bd..43f49bf (verification harness and replay validation work)
```

### Ahead/Behind Analysis

| Branch Pair | Behind | Ahead |
|------------|--------|-------|
| main vs codex/deterministic-core-v1 | 3 | 36 |
| main vs verified-kernel | 3 | 45 |
| codex/deterministic-core-v1 vs verified-kernel | 0 | 9 |

### Common Ancestor Analysis

- **main ↔ codex/deterministic-core-v1**: Common ancestor = `d432abc`
- **main ↔ verified-kernel**: Common ancestor = `d432abc`
- **codex/deterministic-core-v1 ↔ verified-kernel**: Common ancestor = `7dcf3c4` (HEAD of codex/deterministic-core-v1)

**Key Finding**: `verified-kernel` is built entirely on top of `codex/deterministic-core-v1` plus 9 additional commits. The two experimental branches do not have conflicting histories—verified-kernel extends codex/deterministic-core-v1.

---

## 3. Unique Commits per Branch

### Commits Unique to main (not in codex/deterministic-core-v1)

**Total: 3 commits**

```
04362ce - Add files via upload
3ad7d0e - fix: correct format string in adversarial_validation.rs CI BUILD GATE telemetry
70e782a - rewrite project documentation for current verified state
```

**Category**: Documentation and CI fixes; minor corrections

### Commits Unique to codex/deterministic-core-v1 (not in main)

**Total: 36 commits**

```
7dcf3c4 - Commit all current workspace changes
40323b2 - feat: add regulator single-file SDA verification modes and deterministic output
d560a56 - Enforce SDTQ canonical boundary and add sovereign diagnostic attestation
ecc976f - Normalize source headers and add RI04-to-RI18 full-stack shadow-price audit chain
1fe8582 - Add identity-bound attestations, verifier barcode ticketing, and market-condition artifacts
f3d662d - Fix NCR-001 evidence date
... (31 additional commits)
a50992e - Add deterministic SCED hash-chain pipeline (schema-locked, DST-safe)
```

**Category**: Deterministic core implementation, verification infrastructure, compliance harnesses, ERCOT validation

### Commits Unique to verified-kernel (not in codex/deterministic-core-v1)

**Total: 9 commits**

```
43f49bf - docs: add MVRE manual and update performance report; integrate ERCOT normalization and replay equivalence validation
4cb8be6 - Enforce canonical operational snapshot and sovereign trace attestation for deterministic runtime and replay
2309e94 - docs(phase2): Add replay validation completion report and update consolidation status
939fd54 - feat(replay): establish deterministic replay validation harness - CEO-DIR-024
946c10c - feat(consolidation): establish authoritative runtime boundary - CEO-DIR-023-EXEC
21ff1a4 - fix: resolve setpoint guard and ERCOT stress classification test failures
2a9441b - feat(dnp3): add deterministic test-secret auth check and unit test
57b0222 - impl: add DNP3/IEC-61850 helpers and formal admissibility harness; finish remaining directive priorities
49f07bd - Add reserve margin monitoring and adaptive stress governance
```

**Category**: Runtime verification, replay validation, protocol driver enhancements, test fixes

---

## 4. Source File Inventory by Branch

### Files Unique to codex/deterministic-core-v1 (41 new kernel modules)

```
Deterministic Core Infrastructure:
  - src/canonical_core/*.rs (canonicalize, hash, mod, serialize, sort)
  - src/validation_matrix.rs
  - src/sdtq_boundary.rs

Deterministic Verification & Compliance:
  - src/bin/verify_sda.rs
  - src/bin/convert_backcast_profiles.rs
  - src/bin/full_stack_grid_audit.rs
  - src/bin/sced_chain.rs
  - src/bin/timing_probe.rs

ERCOT Integration & Grid Operations:
  - src/sced_offer_chain.rs
  - src/capacity_available_to_sced.rs
  - src/hrly_res_out_cap.rs
  - src/modeling_expectations_policy.rs
  - src/mora_ingestion.rs
  - src/cim_mapping_*.rs (3 files)

Reliability & Control:
  - src/reliability/*.rs (a2e_state_machine, mod, relay_logic)
  - src/reliability_controls.rs
  - src/phase_control.rs
  - src/economics/shadow_prices.rs

Topology & Ingestion:
  - src/topology/*.rs (graph_builder, mod, ybus)
  - src/ingest/rdf_parser.rs

Guardian & Attestation:
  - src/guardian.rs
  - src/sensor_attestation.rs
  - src/tpm_attestation.rs
  - src/trusted_time.rs
  - src/sovereign_diagnostic.rs

Telemetry & State:
  - src/telemetry.rs
  - src/mvre.rs
  - src/sprint1.rs
```

### Files Unique to verified-kernel (7 new kernel modules)

```
Runtime & Verification:
  - src/bin/runtime.rs
  - src/bin/trace_replay.rs
  - src/operational_semantics.rs
  - src/formal_admissibility.rs
  - src/kernel_proofs.rs

ERCOT Integration (Post-codex):
  - src/ercot_ingest.rs
  - src/ercot_stress.rs
```

### Files Shared Between codex/deterministic-core-v1 & verified-kernel (90 files)

**Potential conflict zones** (modified in both branches):
- `src/operator_interface.rs` - Core operator interface
- `src/protocol_drivers.rs` - Protocol driver definitions
- `src/setpoint_guard.rs` - Setpoint guard logic
- `src/sovereign_trace.rs` - Sovereign trace implementation
- `src/bin/pilot_demo.rs` - Pilot demonstration
- `src/topology/graph_builder.rs` - Grid topology builder
- `Cargo.lock`, `Cargo.toml` - Dependency management

---

## 5. File Modification Summary

### Files Modified in codex/deterministic-core-v1 (vs common ancestor)

- **New files**: ~100+ (deterministic core infrastructure)
- **Modified files**: ~50+
- **Large changes**: Cargo.lock, Cargo.toml, README.md, TECHNICAL_SPECIFICATIONS.md, OPERATIONAL_MANUAL.md
- **Data additions**: Grid and Market Conditions/ folder with CSV/PDF artifacts

### Files Modified in verified-kernel (vs codex/deterministic-core-v1)

| Status | Count |
|--------|-------|
| Added | 13 |
| Modified | 7 |
| Deleted | 0 |

**Key modifications**:
- `Cargo.toml`, `Cargo.lock` - New dependencies
- `src/lib.rs` - Module registration
- `src/operator_interface.rs` - Enhanced interface
- `src/protocol_drivers.rs` - Driver updates
- `src/setpoint_guard.rs` - Guard logic refinement
- `src/sovereign_trace.rs` - Trace implementation
- `src/bin/pilot_demo.rs` - Demo updates

---

## 6. Categorization of Changes

### Kernel-Critical Changes

**In codex/deterministic-core-v1**:
- Deterministic core infrastructure (canonical_core module)
- SDTQ boundary enforcement
- Validation matrix and verification scaffolding
- Sovereign diagnostic attestation
- TPM attestation pipeline
- Sovereign trace implementation
- Sensor attestation integration

**In verified-kernel**:
- Deterministic replay validation harness (CEO-DIR-024)
- Authoritative runtime boundary (CEO-DIR-023-EXEC)
- Operational semantics and formal admissibility
- Kernel proofs module
- Runtime binary
- Sovereign trace attestation for replay
- ERCOT stress classification and normalization

### Verification-Related Changes

**In codex/deterministic-core-v1**:
- SCED chain verification (`sced_offer_chain.rs`)
- Full-stack grid audit (`bin/verify_sda.rs`, `bin/full_stack_grid_audit.rs`)
- Validation matrix (`validation_matrix.rs`)
- Timing probe and jitter evidence (`bin/timing_probe.rs`)
- Deterministic telemetry validation
- Compliance ingestion modules
- ERCOT readiness checklist completion

**In verified-kernel**:
- Replay validation completion report (Phase 2)
- Deterministic replay validation harness
- Formal admissibility harness
- Protocol driver verification (DNP3/IEC-61850)

### Non-Conflicting Category Extensions

**codex/deterministic-core-v1** adds entirely new subsystems:
- Deterministic core canonicalization
- Reliability controls and A2E state machine
- Economics module (shadow prices)
- Topology module (graph builder, ybus)
- CIM mapping infrastructure
- Ingest infrastructure (RDF parser)

**verified-kernel** enhances existing subsystems:
- Runtime verification wiring
- Replay validation and trace attestation
- ERCOT stress testing
- Protocol driver enhancements

---

## 7. Potential Merge Conflicts

### High-Confidence Merge Points (No Conflicts Expected)

1. `Cargo.toml` - Both branches extend independently; likely additive
2. `Cargo.lock` - Regenerated during build
3. New modules - No overlap; purely additive (41 + 7 new modules)
4. Documentation files - Non-overlapping updates

### Medium-Confidence Conflict Zones (Manual Review Required)

1. **src/operator_interface.rs** - Modified in verified-kernel; core operator interface
2. **src/protocol_drivers.rs** - Modified in verified-kernel; driver definitions
3. **src/setpoint_guard.rs** - Modified in verified-kernel; guard logic
4. **src/sovereign_trace.rs** - Modified in verified-kernel; trace implementation
5. **src/bin/pilot_demo.rs** - Modified in verified-kernel; demo binary
6. **src/topology/graph_builder.rs** - Modified in verified-kernel; topology builder
7. **src/lib.rs** - Module registration; likely additive

### Low-Conflict Resolution Strategy

- **Shared kernel files (90 total)**: Mostly integration points; merges should favor verified-kernel as it has more recent verification work
- **New modules**: Direct addition; no conflicts
- **Dependencies**: Regenerate lock files post-merge

---

## 8. Integration Readiness Assessment

### Branch Quality Indicators

| Metric | main | codex/deterministic-core-v1 | verified-kernel |
|--------|------|---------------------------|-----------------|
| Commit count | 19 | 52 | 61 |
| Latest activity | 2026-05-21 | 2026-05-13 | 2026-05-22 |
| Purpose | Baseline | Deterministic core | Verification |
| Build status | Unknown* | Unknown* | Unknown* |
| Test status | Unknown* | Unknown* | Unknown* |

*To be determined during Phase 3 post-merge validation

### Recommended Integration Sequence

1. **Phase 3a**: Create backup tags for all branches
2. **Phase 3b**: Merge `origin/codex/deterministic-core-v1` → main
   - 36 kernel-critical commits
   - Deterministic infrastructure and compliance
   - Run `cargo check`, `cargo test`, validation harnesses
3. **Phase 3c**: Merge `origin/verified-kernel` → main
   - 9 verification and runtime commits
   - Runtime boundary and replay validation
   - Run `cargo check`, `cargo test`, verification harnesses

---

## 9. Files Requiring Post-Merge Review

### Critical Integration Files

1. `src/lib.rs` - Ensure all new modules are registered
2. `Cargo.toml` - Dependency resolution
3. `src/operator_interface.rs` - Interface consolidation
4. `src/sovereign_trace.rs` - Trace attestation integration
5. `src/setpoint_guard.rs` - Guard logic unification

### Verification-Critical Files

1. `src/bin/pilot_demo.rs` - Demo integration
2. `src/kernel_proofs.rs` - Verification logic
3. `src/formal_admissibility.rs` - Admissibility harness
4. `src/ercot_ingest.rs` - ERCOT ingest logic

---

## 10. Branch Analysis Summary Table

| Aspect | Finding |
|--------|---------|
| **Total divergence from main** | codex/deterministic-core-v1: 36 ahead; verified-kernel: 45 ahead |
| **Relationship between feature branches** | verified-kernel extends codex/deterministic-core-v1 by 9 commits |
| **New kernel modules** | 41 (codex) + 7 (verified) = 48 new modules total |
| **Shared kernel modules** | 90 files with potential modifications |
| **Conflict risk** | Low to medium; primarily confined to integration points |
| **Merge complexity** | Moderate; sequential merge recommended |
| **Build risk** | Medium; dependencies may need resolution |
| **Verification risk** | Low; both branches contain verification harnesses |

---

## 11. Deliverables Status

- ✅ **Branch inventory report**: Complete
- ✅ **Branch relationship report**: Complete
- ✅ **Unique commit report**: Complete
- ✅ **Diff report**: Complete (file-level)
- 🔲 **Integration plan**: See Section 8
- 🔲 **Post-merge kernel snapshot**: Pending Phase 3
- 🔲 **Conflict report**: See Section 7
- 🔲 **Backup tags**: Pending Phase 3
- 🔲 **Merge execution**: Pending Phase 3

---

## Next Steps: Phase 2 & Phase 3

### Phase 2: Integration Assessment (Detailed Diff Analysis)

Before merging, generate detailed diffs for:
- `src/operator_interface.rs` (codex → verified)
- `src/protocol_drivers.rs` (codex → verified)
- `src/setpoint_guard.rs` (codex → verified)
- `src/sovereign_trace.rs` (codex → verified)
- `Cargo.toml` (codex → verified)

### Phase 3: Controlled Integration

1. Create backup tags: `backup/before-codex-merge`, `backup/before-verified-merge`
2. Merge codex/deterministic-core-v1 into main
3. Resolve conflicts using verified-kernel's newer logic where available
4. Run full validation suite
5. Merge verified-kernel into main
6. Run final validation suite
7. Generate post-merge kernel snapshot

---

**End of Phase 1 Analysis**
