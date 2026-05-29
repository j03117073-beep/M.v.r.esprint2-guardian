# System State Snapshot

## Repository and Version

- Repository: `M.v.r.esprint1-g`
- Branch: `main`
- Commit: `04362ce766d471e1c812c3e56bb95c916be7f7cc`
- Last commit message: `Add files via upload`
- Git working tree: clean

## Build Environment

- Host OS: Ubuntu 24.04.4 LTS
- Rust toolchain: `rustc 1.96.0 (ac68faa20 2026-05-25)`
- Cargo: `cargo 1.96.0 (30a34c682 2026-05-25)`
- default host: `x86_64-unknown-linux-gnu`
- Native packages installed: `pkg-config`, `libtss2-dev`, `build-essential`
- `cargo metadata` output saved at: `target/cargo-metadata.json`
- `cargo tree` output saved at: `target/cargo-tree.txt`

## Dependency State

Declared dependencies in `Cargo.toml`:

- `sha2`
- `ed25519-dalek`
- `tempfile`
- `core_affinity`
- `hex`
- `tss-esapi`
- `serde` (derive)
- `serde_json`
- `axum`
- `tokio` (full)

Resolved dependency graph is present in `Cargo.lock`.

## Kernel / Runtime State

- No kernel process was active before testing.
- Verified runtime by executing the demo CLI and the dashboard server.
- Demo CLI command run successfully: `cargo run --bin demo -- normal`
- Web dashboard command run successfully: `cargo run --bin dashboard`
- Health check endpoint responded: `http://127.0.0.1:3000/health` → `OK`
- The dashboard server was later stopped to leave the workspace clean.

## Build and Execution Results

- `cargo build --workspace`: successful
- `cargo check --workspace`: successful
- `cargo run --bin demo -- normal`: successful
- `cargo run --bin dashboard`: successful

### Demo Output Summary

- `TLBSS DEMO PIPELINE`
- `Scenario: Normal Operation`
- `Admissible: true`
- `✅ No L7 Required`
- `Engine Stable`
- `Audit Clean`
- Execution time reported

### Dashboard Status

- Dashboard server started successfully at `http://127.0.0.1:3000`
- Health endpoint returned `OK`

## Fixes Applied

To complete the build and get runtime verification working, the following code updates were applied:

- Removed injected non-Rust fragment from `src/bin/demo.rs`
- Fixed `src/bin/dashboard.rs` handler usage for `axum`
- Added `serde` [`Serialize`, `Deserialize`] derives for `DemoResult`, `EventType`, `MarketSnapshot`, `FailureAxis`, and `ViolationVector`
- Fixed placeholder initialization in `src/bin/pilot_demo.rs` for `IRModule` and `IRInput`
- Installed required native dependencies for `tss-esapi`

## Known Warnings and Notes

Build completed with warnings only:

- `ICCP_TASE2` non-camel-case enum variant in `src/interface_discovery.rs`
- `sha2::Digest` unused imports in `src/interface_discovery.rs` and `src/operator_interface.rs`
- Unused constants in `src/deployment_manifest.rs`
- Several unused fields and unused functions across `src/hal_output.rs`, `src/interface_discovery.rs`, `src/sovereign_kernel.rs`, and `src/tlbss_integrity_engine.rs`
- `picky-asn1-x509` future incompatibility notice

These warnings do not prevent build or runtime verification but may indicate areas for cleanup.

## Current System Snapshot

- Workspace path: `/workspaces/M.v.r.esprint1-g`
- No dashboard or kernel process is currently left running in this snapshot.
- Build artifacts are available under `target/`
- Runtime verification passed for a representative demo and a web dashboard health check.

## Commands Executed

```bash
curl https://sh.rustup.rs -sSf | sh -s -- -y
. $HOME/.cargo/env
sudo apt-get update
sudo apt-get install -y pkg-config libtss2-dev build-essential
cargo metadata --format-version 1 > target/cargo-metadata.json
cargo tree -e no-dev > target/cargo-tree.txt
cargo check -q --workspace
cargo build --workspace
cargo run --bin demo -- normal
cargo run --bin dashboard
curl -fsS http://127.0.0.1:3000/health
```
