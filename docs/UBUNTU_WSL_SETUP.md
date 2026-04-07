# Ubuntu WSL Setup

Last reviewed: April 7, 2026

This note captures the verified Rust development setup used for M.V.R.ESPRINT1. Latest validated setup run: April 6, 2026.

## Verified Environment

- Windows host with WSL2
- Distribution: `Ubuntu-24.04`
- Rust toolchain:
  - `rustc 1.94.1`
  - `cargo 1.94.1`

## Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

## Install Native Dependencies

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev
```

## Verify the Workspace

```bash
cd /mnt/c/obienova/M.V.R.ESPRINT1
source ~/.cargo/env
cargo check --message-format short
```
