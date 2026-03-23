# M.V.R.ESPRINT1 Operational Manual

**Deterministic Assurance Overlay for Grid Operations**

*Version 0.1.0 – March 2026*

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

M.V.R.ESPRINT1 is a deterministic, cryptographically verifiable assurance layer for energy grid operations. This manual provides guidance for deploying, operating, and maintaining the system in shadow mode for pilot evaluation.

**Key Features**:
- Shadow-mode telemetry consumption
- Deterministic attestation generation
- Tamper-evident audit chains
- External verification capabilities

**Intended Use**: Pilot deployment with no control authority. All operations are observational and analytical.

---

## System Requirements

### Hardware Requirements
- **CPU**: x86_64 or ARM64 architecture, minimum 4 cores
- **RAM**: 8 GB minimum, 16 GB recommended
- **Storage**: 50 GB available space for logs and data
- **Network**: Ethernet connection for telemetry ingestion

### Software Requirements
- **Operating System**: Linux (Ubuntu 20.04+ recommended), or compatible Unix-like system
- **Rust**: Version 1.70+ (installed via rustup)
- **Dependencies**:
  - OpenSSL development libraries
  - TPM 2.0 libraries (for TPM mode, optional)
  - Git for repository access

### Network Requirements
- Access to telemetry sources (ICCP/PMU/SCADA feeds)
- Outbound internet for dependency downloads (during setup)
- Secure communication channels for log transmission

---

## Installation and Setup

### 1. Clone the Repository
```bash
git clone https://github.com/obienova/M.V.R.ESPRINT1.git
cd M.V.R.ESPRINT1
```

### 2. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Install System Dependencies
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev tpm2-tools tpm2-openssl

# For TPM support (optional)
sudo apt install tpm2-abrmd
```

### 4. Build the Project
```bash
cargo build --release
```

### 5. Verify Installation
```bash
cargo test --lib
cargo run --bin zero_state_sanity
```

---

## Configuration

### Environment Variables
Set the following environment variables for operation:

- `SIGNER_MODE`: `simulation` (default) or `tpm` (requires TPM hardware)
- `LOG_LEVEL`: `info` (default), `debug`, or `error`
- `TELEMETRY_ENDPOINT`: URL or path to telemetry source (e.g., `tcp://scada.example.com:2404`)

### Configuration Files
- `Cargo.toml`: Feature flags (e.g., enable `tpm` feature for hardware signing)
- No additional config files required for pilot mode

### Feature Flags
- Default: Simulation mode with software signing
- `tpm`: Enable TPM-backed signing (requires hardware)

Build with TPM support:
```bash
cargo build --release --features tpm
```

---

## Operation

### Starting the System

#### Pilot Demo Mode
```bash
cargo run --bin pilot_demo
```
This generates sample attestation records and verifies the chain.

#### Production Runtime
```bash
SIGNER_MODE=simulation cargo run --bin sovereign_runtime
```

#### With TPM
```bash
SIGNER_MODE=tpm cargo run --bin sovereign_runtime --features tpm
```

### Normal Operation Flow
1. System starts in shadow mode
2. Consumes telemetry from configured sources
3. Generates deterministic traces and attestations
4. Logs are written to `attestation_log.jsonl`
5. Optional: Run verifier for integrity checks

### Stopping the System
- Use `Ctrl+C` to gracefully shut down
- System will complete current attestation cycle before exiting

---

## Monitoring and Logging

### Log Files
- `attestation_log.jsonl`: Append-only attestation records
- Standard output: Runtime logs (configurable via `LOG_LEVEL`)

### Monitoring Commands
```bash
# Check system status
cargo run --bin zero_state_sanity

# Verify recent attestations
cargo run --bin verifier attestation_log.jsonl

# Run grid stability simulation
cargo run --bin tlbss_grid_stability
```

### Key Metrics to Monitor
- Attestation generation rate (should match telemetry frequency)
- Chain verification success (100% expected)
- System resource usage (CPU, memory)
- Telemetry ingestion errors

---

## Maintenance

### Regular Tasks
- **Daily**: Verify attestation chain integrity
- **Weekly**: Run full test suite (`cargo test`)
- **Monthly**: Update dependencies (`cargo update`)
- **Quarterly**: Review and rotate logs

### Log Rotation
Implement log rotation to prevent disk space issues:
```bash
# Example: Rotate logs monthly
logrotate -f /etc/logrotate.d/mvr_esprint1
```

### Backup
- Backup `attestation_log.jsonl` regularly
- Store backups in secure, tamper-evident storage
- Retain logs for regulatory compliance periods

### Updates
```bash
git pull
cargo build --release
# Test before deploying
cargo run --bin rust_simulation_harness
```

---

## Troubleshooting

### Common Issues

#### Compilation Errors
- Ensure Rust 1.70+ is installed
- Check system dependencies: `pkg-config --libs openssl`
- For TPM: Verify TPM hardware and `tpm2-abrmd` service

#### Runtime Errors
- **TPM Unavailable**: Switch to simulation mode or check TPM status
- **Telemetry Connection Failed**: Verify network and endpoint configuration
- **Chain Verification Failed**: Check for log corruption or tampering

#### Performance Issues
- High CPU: Reduce log level or optimize telemetry processing
- Memory usage: Monitor for leaks; restart if necessary
- Disk space: Implement log rotation

### Diagnostic Commands
```bash
# Check TPM status
tpm2_getcap properties-fixed

# Test telemetry connection
telnet <telemetry_host> <port>

# Validate logs
cargo run --bin verifier <log_file>
```

### Escalation
For unresolved issues:
1. Check repository issues on GitHub
2. Review SovereignTrace logs for error details
3. Contact support (see Contact Information)

---

## Safety and Compliance

### Safety Considerations
- System operates in shadow mode only (no control authority)
- All operations are deterministic and auditable
- No unsafe code permitted (`#![deny(unsafe_code)]`)

### Regulatory Compliance
- Designed for NERC BAL-001/002 compliance
- CIP-007/010 integrity protection
- Audit trails suitable for regulatory submission

### Security
- Cryptographic signing of all attestations
- Tamper-evident log chains
- No external network dependencies in runtime
- Secure boot verification available

---

## Contact Information

**Developer**: OBINNA JAMES EJIOFOR
**Email**: [contact information]
**Repository**: https://github.com/obienova/M.V.R.ESPRINT1
**Issues**: https://github.com/obienova/M.V.R.ESPRINT1/issues

For pilot deployment inquiries, please reference the PILOT_BRIEF.md document.

---

*This manual is for pilot evaluation only. Production deployment requires additional validation and certification.*