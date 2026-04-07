# Build Alignment Reference

Version: 0.1.0  
Last validated: April 7, 2026

## Purpose

This file defines which Markdown documents are guaranteed to match the current repository build behavior.

## Build-contract documents

The following are the canonical build-aligned documents:

- `README.md`
- `OPERATIONAL_MANUAL.md`
- `TECHNICAL_SPECIFICATIONS.md`
- `PERFORMANCE_REPORT.md`

## Verified against current build

The following commands were executed successfully on April 7, 2026:

```bash
cargo check --message-format short
cargo run --bin sced_chain -- verify artifacts/sample_sced.csv
cargo run --bin pilot_demo
cargo run --bin verifier pilot_attestation_log.json
cargo run --bin demo -- normal
```

Long-running service check:

```bash
cargo run --bin dashboard
```

Observed bind address: `127.0.0.1:3000` with health endpoint `/health`.

## Notes on other Markdown files

Additional Markdown files in the repository may contain architecture context, planning material, or historical reference notes that are not strict runtime contracts.
