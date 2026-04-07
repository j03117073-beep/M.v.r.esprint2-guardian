# M.V.R.E / sprint1 / Guardian Visual Architecture

Last reviewed: April 7, 2026

## High-Level Topology
```mermaid
flowchart LR
  A["Field and Edge Sources"] --> B["sprint1 Deterministic Binding Layer"]
  B --> C["M.V.R.E Central Control Center"]
  B --> D["Guardian Audit and Integrity Authority"]
  C --> E["Operational Commands and Setpoints"]
  D --> F["Compliance Evidence and Tamper Alerts"]
```

## Responsibility Split
```mermaid
flowchart TB
  subgraph MVRE["M.V.R.E"]
    M1["Supervisory Control"]
    M2["Dispatch and Coordination"]
    M3["Operator Actions"]
  end

  subgraph SPRINT1["sprint1"]
    S1["Schema Lock Enforcement"]
    S2["Deterministic Normalization"]
    S3["Sort, Hash, and Chain Preparation"]
    S4["Dual Delivery to M.V.R.E and Guardian"]
  end

  subgraph GUARDIAN["Guardian"]
    G1["Independent Replay Verification"]
    G2["Chain Integrity Validation"]
    G3["Audit Reports and Evidence Export"]
  end
```

## Deterministic Data Path
```mermaid
sequenceDiagram
  participant Edge as "Field and Edge"
  participant S as "sprint1"
  participant M as "M.V.R.E"
  participant G as "Guardian"

  Edge->>S: Raw SCED records
  S->>S: Validate schema and column order
  S->>S: Normalize and canonicalize values
  S->>S: Deterministic sort by locked key
  S->>S: Compute record_hash and chain_hash
  S->>M: Canonical deterministic records
  S->>G: Same canonical deterministic records
  S->>G: Chain checkpoint and metadata
  G->>G: Independent replay and verification
  G-->>M: Integrity status and audit findings
```

## Verification Decision Flow
```mermaid
flowchart TD
  I["Input Record Batch"] --> V1["Schema Match Check"]
  V1 -->|Fail| E1["CSV_SCHEMA_MISMATCH"]
  V1 -->|Pass| V2["Normalization and Canonicalization"]
  V2 --> V3["Deterministic Sort"]
  V3 --> V4["Primary Key Uniqueness"]
  V4 -->|Fail| E2["DUPLICATE_PK"]
  V4 -->|Pass| V5["Hash and Chain Rebuild"]
  V5 --> V6["Expected Hash and Count Validation"]
  V6 -->|Fail| E3["HASH_MISMATCH or RECORD_COUNT_MISMATCH"]
  V6 -->|Pass| P["PASS"]
```

## Audit Boundary Guarantee
- `M.V.R.E` and `Guardian` consume the same deterministic payload from `sprint1`.
- `Guardian` does not trust operational state from `M.V.R.E`; it replays independently.
- Any single-field change must break deterministic replay and produce explicit failure codes.
