Copyright © 2026 OBINNA JAMES EJIOFOR
All Rights Reserved.

This file is part of the M.V.R.ESPRINT1 Sovereign Execution System,
including TLBSS geometry, the Universal Execution Layer, the
Deterministic IR, Rust Codegen Pipeline, SovereignBus, and the
Cryptographic Audit Chain.

No part of this file, its algorithms, structures, or designs may be
copied, reproduced, modified, distributed, published, sublicensed,
reverse-engineered, or used to create derivative works without the
express written permission of OBINNA JAMES EJIOFOR.

Evaluation and testing by authorized utilities, ISOs/RTOs,
and regulatory bodies is permitted under written agreement.

This software contains proprietary trade secrets and confidential
intellectual property. Unauthorized use is strictly prohibited.


# M.V.R.ESPRINT1

Deterministic Governor for the Energy Sector

A sovereign kernel implementation for energy grid stability and regulatory compliance, built in Rust for maximum safety and determinism.

## Overview

M.V.R.ESPRINT1 implements a comprehensive energy grid governance system with the following key components:

- **Sovereign Kernel**: Core deterministic runtime with 1kHz control loops
- **TLBSS Integrity Engine**: Tri-Entity Boundary Stability System for grid physics modeling
- **Regulatory Policy Engine**: NERC/CIP/ISO compliance with legal citation tracking
- **Projection Operator**: Formal constraint satisfaction with monotonic convergence
- **Audit Guardian**: Non-agentic boundary certification and admissibility checking
- **Adversarial Harness**: Security testing against state corruption and authority escalation
- **Simulation Framework**: Deterministic testing with noise injection and latency simulation
- **Universal Execution Layer**: Cryptographically bound ingestion and execution of multi-language logic
- **Sovereign Bus**: Unified communication channel ensuring all interactions are auditable
- **Sovereign Trace**: Complete cryptographic audit trail from input to output

## Positioning for Grid Deployment

M.V.R.ESPRINT1 is designed as a **deterministic assurance and control overlay** that enhances existing grid infrastructure without operational risk. It focuses on risk reduction and evidence improvement for NERC compliance, not system replacement.

### NERC Mapping and Value Proposition

- **TLBSS Engines**: Support BAL control performance by enforcing constraint envelopes before AGC violations. Provide physics-based guardrails for frequency and voltage stability.
- **Sovereign Kernel (1kHz Loop)**: Ensures deterministic execution, removing timing ambiguity in control actions. Enables replay capability for post-event validation of BAL/PRC performance.
- **Sovereign Bus**: Structured and signed messaging reduces data ambiguity and coordination failures in PRC operations.
- **Sovereign Trace**: Provides cryptographically verifiable event reconstruction for disturbance and misoperation analysis, directly addressing NERC's challenges in proving incident causality.

### Differentiator: Zero-Ambiguity Reconstruction

Unlike traditional systems that optimize control, M.V.R.ESPRINT1 reconstructs every control decision deterministically and proves it hasn't been altered. This enables:

- Replayable event logs for frequency events
- Control decision reconstruction with full input/output traceability
- Tamper-evident audit chains for regulatory compliance

### Phaseable Integration Model

M.V.R.ESPRINT1 integrates incrementally to minimize risk:

1. **Phase 0 - Passive**: Read-only telemetry consumption (ICCP/PMU/SCADA mirror). Generates traces and predicted actions without control authority.
2. **Phase 1 - Advisory**: Outputs recommended AGC setpoints and constraint violations. Operators see "Kernel recommends X".
3. **Phase 2 - Guardrail**: Soft constraints block unsafe commands and flag violations before execution.
4. **Phase 3 - Assisted Control**: Limited closed-loop authority in narrow, well-defined scopes.

### Proposed ERCOT Pilot: Frequency Response Traceability

A shadow-mode deterministic trace engine for frequency events:

- **No Control Authority**: Zero operational risk
- **Existing Telemetry**: Consumes current ICCP/PMU feeds
- **Deliverables**:
  - Replayable event logs for BAL-001 compliance
  - Control decision reconstruction for disturbance analysis
  - Cryptographic proof of log integrity

This pilot demonstrates immediate audit value while establishing a foundation for future control enhancements.

### Example Output Artifact

After processing a simulated frequency event, M.V.R.ESPRINT1 produces verifiable records like:

```json
{
  "event": "frequency_deviation",
  "timestamp": 1710000000,
  "decision": "increase_generation",
  "inputs": {
    "frequency_hz": 59.91,
    "tie_line_mw": 150.0
  },
  "constraints": {
    "bal_001_threshold": 59.94,
    "ramp_rate_limit": 10.0
  },
  "attestation": {
    "decision_hash": "a1b2c3...",
    "pcr_digest": "d4e5f6...",
    "signature": "g7h8i9...",
    "prev_hash": "j0k1l2...",
    "timestamp": 1710000000
  },
  "verification": "valid"
}
```

These records enable zero-ambiguity reconstruction of control decisions and their cryptographic integrity.

### Pilot Demo

Run the included pilot demonstration:

```bash
cargo run --bin pilot_demo
```

This simulates a frequency event sequence, generates attestation records, and verifies the chain integrity, producing output suitable for evaluation by ISO engineers.

## Failure Handling Philosophy

M.V.R.ESPRINT1 does not assume perfect inputs.
Instead, it:

- Records all received telemetry without alteration
- Flags inconsistencies explicitly in SovereignTrace
- Maintains deterministic outputs even under degraded conditions
- Ensures all ambiguity is surfaced, not hidden

This enables operators to distinguish between:
- control failure
- telemetry failure
- external disturbance

## High-Level Architecture Flow

The sovereign kernel operates as a closed-loop regulatory geometry system:

1. **Substrate Input Processing**: `sovereign_runtime` receives external setpoints and sensor data
2. **TLBSS Physics Modeling**: `tlbss_integrity_engine` computes grid stability invariants and plant-class envelopes
3. **Regulatory Policy Application**: `regulatory_policy` maps physical conditions to governance modes (Normal/Degraded/Emergency)
4. **Audit Guardian Certification**: `audit_guardian` performs boundary admissibility checks and vetoes unsafe transitions
5. **Sovereign Trace Generation**: `sovereign_trace` creates immutable audit trails with legal citations

**Harness Integration Points**:
- `adversarial_harness` injects security vectors at the substrate boundary
- `simulation_harness_core` provides deterministic shadow execution for validation
- `zero_state_sanity` validates boot integrity before operational transitions

## Module Interaction Explanation

The system forms a regulatory closed loop where physics, policy, and audit are inseparable:

- **Grid Physics (TLBSS)**: Models resonance curves, saturation limits, and fault-ride-through windows for different plant classes
- **Regulatory Policy**: Translates NERC/CIP/ISO text into executable invariants and governance modes
- **Audit Logic**: Non-agentic certification ensures all transitions maintain system admissibility
- **Sovereign Trace**: Every decision is justified with exact regulatory citations and timestamps

This architecture makes non-compliant states constrained, detected, and
made explicitly auditable through deterministic enforcement.

## Universal Execution Layer

M.V.R.ESPRINT1 can ingest, normalize, and execute logic written in any programming language while maintaining full cryptographic auditability. The Universal Execution Layer is primarily intended for
non-critical logic ingestion, simulation, and advisory workflows.
All production control paths remain deterministic and restricted.

### Cryptographic Input Binding

All inputs entering the system are cryptographically bound through a complete pipeline:

1. **Input Envelope**: Every input is wrapped in a canonical `InputEnvelope` with actor identity, origin language, and raw bytes
2. **Raw Input Hashing**: SHA-256 hash computed before any processing
3. **IR Normalization**: Foreign code converted to deterministic Intermediate Representation (IR)
4. **IR Hashing**: Normalized IR hashed for tamper evidence
5. **Rust Code Generation**: IR compiled to deterministic Rust code
6. **Code Hashing**: Generated Rust code hashed
7. **Execution**: Code executed in sovereign kernel with result hashing
8. **Output Translation**: Results translated back to origin language with final hashing

### Causal Hash Chain

The system maintains a complete cryptographic chain: `raw_input_hash → ir_hash → rust_hash → execution_hash → output_hash`

### Sovereign Bus

All communication flows through the `SovereignBus`, ensuring:
- No actor can bypass audit logging
- All messages are signed or kernel-signed
- Complete traceability of all interactions
- Unified communication for humans, AIs, devices, and subsystems

### Supported Languages

- Python (frontend stub)
- JavaScript (frontend stub)  
- C# (frontend stub)
- Go (frontend stub)
- Rust (native)

This enables M.V.R.ESPRINT1 to serve as a universal deterministic substrate for energy sector logic while providing regulator-grade cryptographic evidence for every operation.

## Architecture

### Core Modules

- `sovereign_kernel`: Main control loop and state management
- `tlbss_integrity_engine`: Grid stability calculations and plant-class invariants
- `regulatory_policy`: Compliance rules and governance modes
- `audit_guardian`: Boundary condition monitoring
- `sovereign_trace`: Immutable audit trail generation
- `simulation_harness_core`: Testing infrastructure
- `adversarial_harness`: Security validation vectors
- `universal_frontend`: Multi-language code ingestion and IR conversion
- `ir_codegen`: Deterministic Rust code generation from IR
- `sovereign_bus`: Unified communication channel for all actors
- `ir_backends`: Output translation back to origin languages
- `crypto_pipeline`: Cryptographic binding of all inputs through execution pipeline

### Binaries

- `sovereign_runtime`: Production kernel executable
- `tlbss_grid_stability`: Grid stability simulation
- `rust_simulation_harness`: Commissioning validation
- `zero_state_sanity`: Boot integrity checks

## Development Workflow

Expected engineering cycle for M.V.R.ESPRINT1 development:

1. **Modify Module**: Update kernel components or regulatory mappings
2. **Run Unit Tests**: Execute `cargo test --lib` for canonical validation (currently 9/9 passing)
3. **Run Canonical Harness**: Validate 100-tick compliance scenarios
4. **Run Adversarial Harness**: Test security vectors against state corruption
5. **Run Commissioning/Grid Sims**: Execute `zero_state_sanity` and `tlbss_grid_stability`
6. **Inspect SovereignTrace**: Review audit trails for regulatory compliance
7. **Commit + Push**: Ensure repository reflects validated state

## Regulatory-Geometry Integration

The system compiles regulatory text into executable geometry:

**NERC → Invariants**:
- BAL-001/002: Frequency/ACE stability encoded in TLBSS coherence thresholds
- PRC-005/023: Protection relay rules mapped to boundary condition checks
- FAC-008/014: Facility ratings translated to per-node saturation limits

**CIP → Immutability + Trace Integrity**:
- CIP-007: System integrity enforced via deterministic state transitions
- CIP-010: Configuration integrity maintained through SovereignTrace
- CIP-013: Trust boundaries protected by audit guardian admissibility checks

**ISO/RTO → Dispatch Constraints**:
- Market rules compiled into contractual manifolds
- Ramp-rate limits enforced as monotonic charge constraints
- Reserve margins encoded as global admissibility conditions

**Plant-Class → Physics Invariants**:
- Solar/Wind/Hydro plants have unique resonance profiles
- Saturation curves prevent over-generation
- FRT windows ensure ride-through capability
- Ramp envelopes constrain operational flexibility

## Regulatory Compliance

The system integrates with major energy regulatory frameworks:

- **NERC**: Reliability standards (BAL-001/002, PRC-005/023, FAC-008/014)
- **CIP**: Cybersecurity standards (CIP-007, CIP-010, CIP-013)
- **ISO/RTO**: Market and operational standards
- **Plant-Class Codes**: Physics-based invariants for different energy sources

## Building and Testing

```bash
# Build the project
cargo build --release

# Run unit tests
cargo test --lib

# Run commissioning harnesses
cargo run --bin zero_state_sanity
cargo run --bin rust_simulation_harness
cargo run --bin tlbss_grid_stability
```

## Roadmap / Next Steps

- **Regulatory DSL**: Domain-specific language for compiling policy text → invariants
- **Expanded PlantClass Envelopes**: Complete resonance curves and FRT windows for all plant types
- **Distributed Transition Mesh**: Listener-side components for multi-node coordination
- **SovereignTrace v2**: Richer regulator evidence with compressed audit trails
- **Extended Adversarial Vectors**: Additional security scenarios and fuzzing

## Safety

- `#![deny(unsafe_code)]` enforced throughout
- Deterministic execution with bounded latency
- Immutable state transitions
- Comprehensive audit trails

## License

[License information here]
