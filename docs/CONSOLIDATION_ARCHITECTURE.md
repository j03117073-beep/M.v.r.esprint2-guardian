# MVRE Consolidation Phase Architecture

**CEO-DIR-023-EXEC** | **Status: ACTIVE** | **Date: 2026-05-19**

---

## Executive Summary

The MVRE system has achieved sufficient architectural mass to proceed from expansion into **formal consolidation and operational definition**.

The repository has been restructured into **three distinct, formally separated execution boundaries**:

1. **Authoritative Runtime** - Production/operational path
2. **Formal Verification Runtime** - Proof validation and compliance
3. **Research / Adversarial Sandbox** - Experimentation and demonstration

This document defines the architectural boundaries, governance model, and operational deployment strategy.

---

## I. The Canonical Operational Spine

The MVRE system implements a deterministic infrastructure pipeline:

```
Telemetry Ingestion
    ↓
Protocol Validation (DNP3, IEC-61850, ICCP-TASE2, C37.118, Modbus)
    ↓
Canonicalization (Protocol → PowerState → Trajectory)
    ↓
Admissibility Arbitration (Constraint Evaluation)
    ↓
Deterministic Kernel Arbitration (SovereignKernel)
    ↓
Sovereign Trace Generation (Unfalsifiable Audit Chain)
    ↓
Operator / Regulator Visibility (Dashboard, Compliance Export)
```

This pipeline represents the **complete operational flow** for grid security and market operations.

Every component in this pipeline:
- Enforces deterministic computation boundaries
- Produces auditable decision traces
- Resists manipulation or autonomous override
- Generates regulatory compliance evidence

---

## II. The Three Execution Boundaries

### A. AUTHORITATIVE RUNTIME

**File**: `src/bin/runtime.rs`

**Purpose**: The single canonical operational execution path for MVRE.

**Entry Point**:
```bash
cargo run --bin runtime
```

**Operational Phases**:
1. Telemetry Ingestion
2. Protocol Validation
3. Canonicalization
4. Admissibility Arbitration
5. Deterministic Kernel Arbitration
6. Sovereign Trace Generation
7. Operator / Regulator Visibility

**Governance**:
- This is the **deployment boundary**
- This is the **production runtime**
- This defines the **authoritative operational semantics**
- This is the path that utilities and regulators certify

**Core Modules**:
- `sovereign_kernel.rs` - Center of gravity
- `constraint_system.rs` - Admissibility evaluation
- `sovereign_trace.rs` - Unfalsifiable audit
- `operator_interface.rs` - Operator visibility
- `protocol_drivers.rs` - Protocol authentication

**Guarantees**:
- Deterministic execution within bounded ticks
- Perfect auditability via sovereign traces
- No autonomous override of admissibility checks
- Safe-state transition logic

**Configuration**:
- `MVRE_MAX_TICKS` - Kernel tick limit (default: 100)
- `MVRE_TELEMETRY_STALENESS_MS` - Telemetry freshness threshold (default: 20000)
- `MVRE_ARTIFACTS_DIR` - Audit artifact output (default: ./mvre-artifacts)
- `MVRE_MODE` - Operating mode (operational, shadow, diagnostic)
- `MVRE_CYCLES` - Execution cycles per run (default: 1)

---

### B. FORMAL VERIFICATION RUNTIME

**Files**: 
- `src/bin/verifier.rs`
- `src/bin/formal_proof_harness.rs`

**Purpose**: Prove properties, validate reproducibility, and verify regulatory compliance.

**Entry Points**:
```bash
cargo run --bin verifier <artifact_log>
cargo run --bin formal_proof_harness
```

**Verification Scope**:
- Determinism proofs via Kani formal verification
- Admissibility consistency validation
- Replay assurance (same input → same output)
- Timing coherence validation
- Protocol authenticity enforcement
- TPM-chain integrity

**Role**:
- Assurance activity (not operational)
- Regulators use this to audit MVRE
- Operators use this to validate kernel behavior
- CI/CD uses this to gate releases

**Guaranteed Properties**:
- No path violates admissibility
- All decisions traceable to legal citations
- Time-coupling constraints satisfied
- TPM chains unfalsifiable

---

### C. RESEARCH / ADVERSARIAL SANDBOX

**Files**:
- `src/bin/demo.rs` - Demonstration scenarios
- `src/bin/pilot_demo.rs` - Pilot kernel execution
- `src/bin/adversarial_harness.rs` - Attack injection
- `src/bin/simulation.rs` - System simulation

**Purpose**: Safe-space experimentation, stress testing, and adversarial validation.

**Entry Points**:
```bash
cargo run --bin demo <scenario>
cargo run --bin pilot_demo
cargo run --bin adversarial_harness
cargo run --bin simulation
```

**Scope**:
- Market stress scenarios (normal, reserve shortage, capacity shortage, etc.)
- Adversarial injection (reference corruption, feedback instability, coupling violations)
- Failure axis testing (FailureAxis enum exhaustion)
- Bench-scale demonstrations

**Explicitly NOT Operational**:
- These binaries cannot be confused with production paths
- They generate scenario artifacts, not audit traces
- They are isolated from the canonical runtime boundary

**Use Cases**:
- Engineer training and scenario exploration
- Regulatory demonstration events
- Stress testing in controlled environments
- Adversarial validation of defense mechanisms

---

## III. Module Organization

```
src/
├── bin/
│   ├── runtime.rs                 ← AUTHORITATIVE RUNTIME
│   ├── verifier.rs                ← VERIFICATION
│   ├── formal_proof_harness.rs    ← VERIFICATION
│   ├── demo.rs                    ← RESEARCH
│   ├── pilot_demo.rs              ← RESEARCH
│   ├── adversarial_harness.rs     ← RESEARCH
│   ├── simulation.rs              ← RESEARCH
│   └── dashboard.rs               ← OPERATOR TOOLING
│
├── core/
│   ├── sovereign_kernel.rs        ← Kernel arbitration
│   ├── kernel.rs                  ← Kernel state machine
│   ├── constraint_system.rs       ← Admissibility evaluation
│   └── formal_admissibility.rs    ← Formal proofs
│
├── audit/
│   ├── sovereign_trace.rs         ← Unfalsifiable trace
│   ├── audit_guardian.rs          ← Trace enforcement
│   └── testament_audit.rs         ← Post-execution audit
│
├── protocols/
│   ├── protocol_drivers.rs        ← Protocol validation
│   ├── interface_discovery.rs     ← Endpoint discovery
│   └── drivers/
│       ├── dnp3/
│       ├── iec61850/
│       ├── modbus/
│       └── cim/
│
├── telemetry/
│   ├── telemetry.rs              ← Telemetry validation
│   ├── sensor_attestation.rs     ← Sensor binding
│   └── tpm_attestation.rs        ← TPM anchoring
│
├── operators/
│   ├── operator_interface.rs     ← Operator visibility
│   ├── dashboard.rs              ← Dashboard UI
│   └── regulatory_policy.rs      ← Compliance encoding
│
└── research/
    ├── demo_pipeline.rs          ← Scenario execution
    ├── adversarial_harness.rs    ← Attack injection
    ├── simulation_harness_core.rs ← Simulation engine
    └── failure_axis.rs           ← Fault injection
```

---

## IV. Deployment Strategy

### Phase 1: Pilot Deployment (Current)

- **Path**: `cargo run --bin runtime`
- **Scope**: Single-machine deterministic execution
- **Telemetry**: Simulated measurement points
- **Signer**: Simulated TPM (no hardware requirement)
- **Artifacts**: JSON attestation logs

### Phase 2: Hardware-Backed Deployment

- **TPM**: Real TPM 2.0 (via tss-esapi)
- **Telemetry**: Real protocol ingestion (DNP3, IEC-61850)
- **Signer**: Hardware TPM attestation
- **Artifacts**: Cryptographically sealed audit chains

### Phase 3: Distributed Deployment

- **Multiple kernels**: Coordinated across substations
- **SovereignBus**: Inter-kernel communication
- **Consensus**: Deterministic voting on state transitions
- **Scalability**: Horizontal kernel placement

---

## V. Governance and Invariants

### Release-Critical Invariants (Sealed)

No subsystem may:

1. **Bypass admissibility** - Every decision must pass constraint evaluation
2. **Violate deterministic execution** - Same input → Same output (provable)
3. **Suppress sovereign traces** - Every decision generates an unfalsifiable audit record
4. **Introduce uncontrolled autonomous behavior** - Kernel arbitration is bounded and auditable

### Ambiguity Resolution

When ambiguity arises:

- Resolve toward **safety** (conservative action)
- Resolve toward **non-action** (default deny)
- Resolve toward **audit escalation** (regulator visibility)
- Resolve toward **Q-state isolation** (emergency containment)

---

## VI. Verification and Certification Pathway

### Phase 1: Internal Validation

```
cargo build --bin runtime        # Compile
cargo test                        # Unit tests
cargo run --bin formal_proof_harness  # Kani proofs
cargo run --bin verifier          # Replay validation
```

### Phase 2: Regulatory Audit

1. Provide `runtime.rs` source code
2. Provide formal proof report (`KANI_KERNEL_VERIFICATION_REPORT.md`)
3. Provide sovereign traces from pilot executions
4. Provide protocol authentication evidence
5. Provide TPM anchor certificates

### Phase 3: Certification

MVRE becomes certified as:

> A Deterministic Operational Trust Kernel for Critical Infrastructure Systems

---

## VII. Documentation by Boundary

### Authoritative Runtime Documentation
- [OPERATIONAL_MANUAL.md](../OPERATIONAL_MANUAL.md) - Deployment guide
- [PERFORMANCE_REPORT.md](../PERFORMANCE_REPORT.md) - Timing validation
- Runtime configuration and tuning

### Verification Documentation
- [KANI_KERNEL_VERIFICATION_REPORT.md](../docs/KANI_KERNEL_VERIFICATION_REPORT.md)
- Formal proof artifacts
- Replay validation reports

### Research / Demonstration Documentation
- [PILOT_BRIEF.md](../PILOT_BRIEF.md) - Scenario descriptions
- [PILOT_TEST_MATRIX.md](../docs/PILOT_TEST_MATRIX.md) - Test cases
- Adversarial attack library
- Stress scenario definitions

---

## VIII. What Changed

### Before Consolidation

- Multiple execution paths without clear authority
- Ambiguity between operational, verification, and demo code
- Unclear deployment boundary
- Mixed concerns across binaries

### After Consolidation

✅ **One authoritative runtime** (`runtime.rs`)
✅ **One verification path** (`verifier.rs`, `formal_proof_harness.rs`)
✅ **One research sandbox** (demo, pilot, adversarial, simulation)
✅ **Clear operational authority** (SovereignKernel as center of gravity)
✅ **Explicit governance boundaries** (no ambiguity in deployment)
✅ **Perfect auditability** (sovereign traces for every decision)

---

## IX. Next Steps

1. ✅ **Define the authoritative runtime** (DONE - `runtime.rs`)
2. ⏳ **Establish operational procedures** (ERCOT commissioning workflow)
3. ⏳ **Produce deployment checklist** (hardware, TPM, protocol endpoints)
4. ⏳ **Execute pilot with real telemetry** (live grid conditions)
5. ⏳ **Validate formal proofs** (Kani harness on full kernel)
6. ⏳ **Prepare regulatory documentation** (compliance evidence)

---

## X. Authority and Governance

**SIGNED:**

```
Obinna James Ejiofor
CEO & Orchestrator
MVRE Program
Date: 2026-05-19
Status: CONSOLIDATION PHASE ACTIVE
Authority: CEO-DIR-023-EXEC
```

**All Release State Alpha invariants remain sealed and binding.**

---

## Appendix A: Binary Matrix

| Binary | Purpose | Classification | Deployment | Certification |
|--------|---------|-----------------|------------|----------------|
| `runtime` | Operational kernel | Authoritative | Production | Required |
| `verifier` | Proof validation | Verification | CI/Audit | Required |
| `formal_proof_harness` | Formal proofs | Verification | CI/Release | Required |
| `demo` | Scenario execution | Research | Training | Optional |
| `pilot_demo` | Kernel demo | Research | Training | Optional |
| `adversarial_harness` | Attack injection | Research | Testing | Optional |
| `simulation` | System simulation | Research | Testing | Optional |
| `dashboard` | Operator UI | Tooling | Operations | Recommended |

---

## Appendix B: Environment Variables

```bash
# Runtime Configuration
MVRE_MAX_TICKS=100                           # Kernel execution bound
MVRE_TELEMETRY_STALENESS_MS=20000            # Telemetry freshness
MVRE_ARTIFACTS_DIR=./mvre-artifacts          # Audit output
MVRE_MODE=operational                        # {operational, shadow, diagnostic}
MVRE_CYCLES=1                                # Cycles per execution

# Cryptography
SIGNER_MODE=simulation                       # {simulation, tpm}

# Logging
RUST_LOG=info                                # Log level
RUST_BACKTRACE=1                             # Error tracing
```

---

**END OF CONSOLIDATION ARCHITECTURE DOCUMENT**
