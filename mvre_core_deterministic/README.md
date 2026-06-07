mvre_core_deterministic
=======================

Core deterministic execution primitives for MVRE.

Provides:
- `ExecutionCommitment` canonical representation and `to_bytes()` for deterministic serialization
- `DeterministicExecutable` trait for execution-bound modules
- `Transaction` scratchpad and deterministic `commit()` producing commitments

Usage:

Add as a dependency in `Cargo.toml`:

```toml
mvre_core_deterministic = { path = "mvre_core_deterministic" }
```

Run tests with `cargo test -p mvre_core_deterministic`.
