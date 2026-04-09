# Build Environment Guidance (Appendix)

## Canonical Build Path (Submission/Repro)

- Use a native Linux filesystem (RHEL/Ubuntu).
- Pin toolchain with `rust-toolchain.toml`.
- Build: `cargo build --release`.
- Capture binary hashes and a minimal smoke test log.

## Developer Convenience Path (Windows + WSL2)

- WSL2 on Windows is acceptable, but `/mnt/c/...` builds are slower.
- For faster runs, copy repo to `~/M.V.R.ESPRINT1` and build there.

## Evidence Checklist

- Release build log (`cargo build --release`).
- SHA-256 hashes for release binaries.
- Smoke test result:
  - `target/release/sced_chain.exe verify test_vectors/gold_truth_sced_20260322_1805.csv`

