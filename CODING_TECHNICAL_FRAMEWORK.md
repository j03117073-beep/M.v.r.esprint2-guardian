# Coding Technical Framework

This document outlines the technical framework and best practices for developing, testing, and maintaining the **M.V.R. ESPRINT1** codebase.

## 1. Goals

- Ensure **safety**, **reliability**, and **traceability** for all code changes.
- Enable **consistent code quality** across contributors.
- Provide a clear baseline for **review**, **testing**, and **deployment**.

---

## 2. Repository Structure

- **`src/`**: Main Rust code and core libraries.
- **`src/bin/`**: Executable entrypoints (CLI binaries, demos, harnesses).
- **`src/drivers/`**: Hardware/driver abstractions.
- **`src/sp_api/`**: Protocol/interface API boundaries.
- **`artifacts/`**: Generated artifacts, manifests, and configuration outputs.
- **`target/`**: Build outputs (ignored by VCS).

> 🧠 Keep high-level logic in `src/` and avoid placing generated artifacts in version control.

---

## 3. Coding Standards (Rust)

### 3.1 Formatting

- Use **`cargo fmt`** to format code consistently.
- Add a pre-commit hook or CI step to enforce formatting.

### 3.2 Linting

- Use **`cargo clippy`** to catch common mistakes and enforce idiomatic Rust patterns.
- Treat `clippy::pedantic` suggestions as a strong recommendation; document exceptions when needed.

### 3.3 Documentation

- Document public APIs using **`///`** doc comments.
- Maintain high-level design docs in `*.md` (e.g., `ARCHITECTURE_DESIGN.md`).
- Update docs when making behavior or API changes.

---

## 4. Testing Strategy

### 4.1 Unit Tests

- Place unit tests in the same module as the code under test using `#[cfg(test)]`.
- Keep tests deterministic and fast.

### 4.2 Integration Tests

- Add integration tests under `tests/` (create if not present).
- Cover end-to-end behavior of public APIs.

### 4.3 Simulation & Harness Tests

- Use harness binaries under `src/bin/` for simulation scenarios and safety validation.
- Keep harnesses focused and repeatable.

---

## 5. CI / Automation

### 5.1 Build

- Ensure `cargo build --all --tests` passes on every CI run.

### 5.2 Test

- Run `cargo test --all` on every pull request.
- Include any required environment setup in CI config.

### 5.3 Security

- Check dependencies for vulnerabilities (e.g., `cargo audit`).
- Keep third-party dependencies up to date and pinned.

---

## 6. Version Control & Workflows

### 6.1 Branching

- Use feature branches for development work.
- Keep `main` stable and deployable.

### 6.2 Pull Requests

- Provide a clear PR description, summary of changes, and how to test.
- Ensure 1-2 reviewers review all code changes.

### 6.3 Commit Messages

- Use clear, present-tense commit messages.
- Prefer small, self-contained commits.

---

## 7. Safety & Compliance

- Prefer explicit error handling and avoid `unwrap()`/`expect()` in production paths.
- Use the project’s existing audit and integrity tooling (e.g., `audit_guardian.rs`, `tlbss_integrity_engine.rs`).

---

## 8. Extending the Framework

When adding new subsystems or interfaces, update this document to include:
- New module patterns or directory conventions.
- Required CI checks and test coverage expectations.
- Any special build targets or release artifacts.
