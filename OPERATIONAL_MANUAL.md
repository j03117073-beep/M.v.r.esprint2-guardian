# M.V.R.ESPRINT1 Operational Manual

**Deterministic Assurance Overlay for Grid Operations**

*Version 0.1.0 - April 2026*

---

## Table of Contents

1. [Introduction](#introduction)
2. [System Requirements](#system-requirements)
3. [Installation and Setup](#installation-and-setup)
4. [Configuration](#configuration)
5. [Operation](#operation)
6. [Monitoring and Logging](#monitoring-and-logging)
7. [Maintenance](#maintenance)
8. [Troubleshooting](#troubleshooting)
9. [Safety and Compliance](#safety-and-compliance)
10. [Contact Information](#contact-information)

---

## Introduction

M.V.R.ESPRINT1 is a deterministic, cryptographically verifiable assurance layer for energy-grid evaluation workflows. This manual documents the current operator-facing commands and verification paths present in the repository.

**Key Features**:
- Deterministic attestation generation
- Tamper-evident audit chains
- External verification of attestation logs
- Deterministic CSV normalization and chain verification
- Demo and dashboard entry points for pilot evaluation

**Intended Use**: Pilot evaluation and deterministic verification workflows. The currently exposed operator-facing binaries are `pilot_demo`, `verifier`, `sced_chain`, `demo`, `dashboard`, and `formal_proof_harness`.

---

## System Requirements

### Hardware Requirements
- **CPU**: x86_64 or ARM64 architecture, minimum 4 cores
- **RAM**: 8 GB minimum, 16 GB recommended
- **Storage**: 50 GB available space for logs and artifacts

### Software Requirements
- **Operating System**: Linux recommended; Ubuntu 24.04 under WSL2 is the documented verified setup
- **Rust**: Verified toolchain is `rustc 1.94.1` / `cargo 1.94.1`
- **Dependencies**:
  - `build-essential`
  - `pkg-config`
  - `libssl-dev`
  - TPM 2.0 libraries only when building with the optional `tpm` feature
  - Git for repository access

### Verified Environment
- **Workspace path on WSL**: `/mnt/c/obienova/M.V.R.ESPRINT1`
- **Reference setup note**: [`docs/UBUNTU_WSL_SETUP.md`](docs/UBUNTU_WSL_SETUP.md)

---

## Installation and Setup

### 1. Clone the Repository
```bash
git clone https://github.com/obienova/M.V.R.ESPRINT1.git
cd M.V.R.ESPRINT1
```

### 2. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### 3. Install System Dependencies
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev
```

### 4. Optional TPM Dependencies
```bash
sudo apt-get install -y tpm2-tools tpm2-openssl tpm2-abrmd
```

### 5. Build the Project
```bash
cargo build --release
```

### 6. Verify Installation
```bash
cargo check --message-format short
cargo run --bin sced_chain -- verify artifacts/sample_sced.csv
cargo run --bin pilot_demo
```

---

## Configuration

### Environment Variables
Set the following environment variable when running signer-dependent flows:

- `SIGNER_MODE`: `simulation` or `tpm`

`pilot_demo` sets `SIGNER_MODE=simulation` internally before invoking the kernel path. For direct signer selection in your shell, export the variable before running a binary that calls `signer_from_env()`.

### Configuration Files
- `Cargo.toml`: dependency and feature configuration, including optional `tpm`
- `artifacts/sample_sced.csv`: sample deterministic verification input for `sced_chain`
- `dashboard.html`: static dashboard page served by the Axum dashboard binary

### Feature Flags
- Default: software-backed evaluation paths
- `tpm`: enable TPM-backed signing code paths

Build with TPM support:
```bash
cargo build --release --features tpm
```

---

## Operation

### Running the Current Tooling

#### Pilot Demo
```bash
cargo run --bin pilot_demo
```
This generates `pilot_attestation_log.json` and immediately invokes `verifier` on the generated file.

#### Verifier
```bash
cargo run --bin verifier pilot_attestation_log.json
```
This validates a JSON array of `AttestationRecord` values.

#### SCED Chain Verification
```bash
cargo run --bin sced_chain -- verify artifacts/sample_sced.csv
```
This performs deterministic CSV parsing, schema validation, sorting, chain reconstruction, and status output.

#### Demo Scenarios
```bash
cargo run --bin demo -- normal
cargo run --bin demo -- all
```

#### Dashboard
```bash
cargo run --bin dashboard
```
The dashboard listens on `http://localhost:3000` and exposes a health check at `/health`.

#### TPM-Capable Build
```bash
cargo build --release --features tpm
```
The TPM path is optional and parts of the TPM signer remain scaffolded.

### Normal Operation Flow
1. Verify the workspace with `cargo check --message-format short`
2. Run `sced_chain` against a CSV input when validating deterministic market artifacts
3. Run `pilot_demo` when generating sample attestation records and verifier handoff output
4. Run `verifier` against a JSON attestation log when checking chain integrity directly
5. Use `demo` or `dashboard` for scenario walkthroughs and visualization

### Stopping the System
- Use `Ctrl+C` to stop any currently running binary
- For `dashboard`, the local Axum server exits immediately after interrupt

---

## Monitoring and Logging

### Log Files and Outputs
- `pilot_attestation_log.json`: sample attestation log produced by `pilot_demo`
- Standard output: verifier, chain-verification, and demo status output
- Dashboard health endpoint: `GET /health`

### Monitoring Commands
```bash
# Verify an attestation log
cargo run --bin verifier pilot_attestation_log.json

# Verify deterministic CSV chain behavior
cargo run --bin sced_chain -- verify artifacts/sample_sced.csv

# Start dashboard
cargo run --bin dashboard
```

### Key Metrics to Monitor
- Chain verification success
- Deterministic final hash stability across repeated runs
- Verifier mismatch index and error code output
- Dashboard availability on port `3000`

---

## Maintenance

### Regular Tasks
- **Per change**: Verify with `cargo check --message-format short`
- **Daily or before demos**: Run `cargo run --bin sced_chain -- verify artifacts/sample_sced.csv`
- **When validating attestations**: Run `cargo run --bin pilot_demo` and re-run `verifier` if needed
- **Periodically**: Review generated artifacts and remove stale demo outputs

### Backup
- Backup generated attestation logs when they are used as demo artifacts
- Store validation artifacts in tamper-evident storage if they are used for external review

### Updates
```bash
git pull
cargo build --release
cargo check --message-format short
cargo run --bin sced_chain -- verify artifacts/sample_sced.csv
```

---

## Troubleshooting

### Common Issues

#### Compilation Errors
- Ensure the documented Rust toolchain is installed
- Check system dependencies: `pkg-config --libs openssl`
- For TPM: verify the optional TPM packages and feature flag

#### Runtime Errors
- **TPM Unavailable**: Switch to simulation mode or verify TPM device and feature enablement
- **Verifier Failed**: Check for malformed JSON, corrupted log files, or broken hash linkage
- **CSV Verification Failed**: Check schema, duplicate primary keys, invalid booleans, invalid numerics, or expected-hash mismatch

#### Performance Issues
- High CPU: stop unused local binaries, especially the dashboard server
- Memory usage: restart the active binary and verify no extra local processes are running
- Disk space: remove stale artifacts such as old generated logs if they are no longer needed

### Diagnostic Commands
```bash
# Validate attestation log
cargo run --bin verifier <log_file>

# Validate SCED CSV
cargo run --bin sced_chain -- verify <input.csv> [expected_hash]

# Check TPM status when using TPM-backed flows
tpm2_getcap properties-fixed
```

### Escalation
For unresolved issues:
1. Review the failing command output carefully
2. Re-run `cargo check --message-format short`
3. Compare the input artifact against `artifacts/sample_sced.csv`
4. Review repository issues or project docs for the affected workflow

---

## Safety and Compliance

### Safety Considerations
- Current exposed binaries are evaluation and verification tools
- No production control runtime binary is present in `src/bin`
- All reviewed workflows are deterministic and auditable
- No unsafe code permitted (`#![deny(unsafe_code)]`)

### Regulatory Compliance
- Designed around deterministic evidence generation and verification
- Supports review against NERC/CIP-oriented assurance goals documented elsewhere in the repository
- Generated artifacts are suitable for pilot evaluation, not automatic operational control

### Security
- Cryptographic signing of attestation records
- Tamper-evident log chains
- Optional TPM-backed signer path via Cargo feature
- Some execution-layer and TPM internals remain scaffolded and should be treated as non-production

---

## Contact Information

**Developer**: OBINNA JAMES EJIOFOR  
**Repository**: https://github.com/obienova/M.V.R.ESPRINT1  
**Issues**: https://github.com/obienova/M.V.R.ESPRINT1/issues

For pilot deployment context, also review `PILOT_BRIEF.md`, `README.md`, and the docs in `docs/`.

---

## April 2026 Ubuntu WSL Addendum

### Verified Environment

- Ubuntu `24.04` on WSL2
- `rustc 1.94.1`
- `cargo 1.94.1`
- Native dependencies: `build-essential`, `pkg-config`, `libssl-dev`

### Recommended Verification Commands

```bash
cargo check --message-format short
cargo run --bin sced_chain -- verify artifacts/sample_sced.csv
cargo run --bin pilot_demo
cargo run --bin dashboard
```

### WSL Recovery

If Ubuntu stops responding after a reboot or interrupted Cargo run:

```powershell
wsl --shutdown
wsl -d Ubuntu-24.04
```

If Cargo leaves a stale build lock after interruption:

```powershell
Remove-Item -LiteralPath C:\obienova\M.V.R.ESPRINT1\target\debug\.cargo-lock -Force
```
