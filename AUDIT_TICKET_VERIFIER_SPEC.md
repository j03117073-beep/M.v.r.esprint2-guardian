# Audit Ticket Verifier Specification

## Purpose

This specification defines the deterministic audit ticket manifest and verification engine for the M.V.R.ESPRINT1 repository.
The verifier validates repository identity, certification provenance, artifact hashes, and audit ticket structure.

## Components

### 1. Audit Manifest

The audit manifest captures the repository state at certification time. It includes:

- `repository`: current repository name, commit, and tag
- `baseline`: the certified baseline commit and tag
- `generated_at`: Unix timestamp when the manifest was generated
- `artifacts`: array of repository file records containing path, SHA256, and size
- `manifest_hash`: SHA256 of the canonical manifest content excluding the `manifest_hash` field

### 2. Audit Ticket

The audit ticket formalizes the certification artifact.
It contains:

- `ticket_version`: schema version, currently `1.0`
- `manifest`: the captured audit manifest
- `ticket_signature`: deterministic signature over the manifest hash, baseline commit, and baseline tag
- `summary`: human-readable summary of the certification purpose

### 3. Signature Requirements

The ticket signature is computed using SHA256 over the following canonical input:

- constant seed string: `M.V.R.ESPRINT1-AUDIT-TICKET-V1`
- `manifest_hash`
- `baseline.commit`
- `baseline.tag`

The signer is deterministic and derived from the audit ticket contents.
This prevents accidental ticket tampering and enables independent verification of ticket integrity.

### 4. Verification Outputs

The verifier produces a JSON report with:

- `outcome`: one of `PASS`, `PASS WITH CONDITIONS`, or `FAIL`
- `details`: list of verification observations
- `baseline_commit`: expected baseline commit from the ticket
- `baseline_tag`: expected baseline tag from the ticket
- `mismatches`: artifact hash mismatches
- `missing_files`: repository files referenced by the manifest but not found locally

## Verification Engine Behavior

The verifier performs the following checks:

1. Validate manifest structure and parse the ticket JSON
2. Recompute the manifest hash and compare with `manifest.manifest_hash`
3. Recompute the ticket signature and compare with `ticket_signature`
4. Verify repository identity using local Git metadata
5. Verify the manifest artifact entries against actual repository files
6. Determine final outcome:
   - `PASS` when all checks succeed
   - `PASS WITH CONDITIONS` when repository identity is incomplete but hash checks pass
   - `FAIL` when any structural, signature, or file hash check fails

## File Traversal Rules

The manifest generator traverses the repository tree recursively and excludes:

- `.git`
- `target`
- `node_modules`

Artifact paths are stored in canonical forward-slash form.

## Repository Identity

Repository identity is sourced from Git:

- commit: `git rev-parse --short HEAD`
- tag: `git tag --points-at HEAD`

This ensures the verifier can confirm the local checkout matches the certified baseline.
