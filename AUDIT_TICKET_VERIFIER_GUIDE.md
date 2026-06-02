# Audit Ticket Verifier Execution Guide

## Overview

This guide describes how an independent reviewer can generate and verify the cryptographic audit ticket using only the repository contents.

## Available Binary

Build the verifier binary with:

```bash
cargo build --bin audit_ticket_verifier
```

## Generating a Repository Manifest

Create an audit manifest of the repository state:

```bash
cargo run --bin audit_ticket_verifier -- manifest-generate audit_manifest.json
```

This writes `audit_manifest.json` containing:

- repository identity
- baseline commit and tag
- file-level SHA256 hashes
- canonical manifest hash

## Creating an Audit Ticket

Generate a signed audit ticket from the manifest:

```bash
cargo run --bin audit_ticket_verifier -- ticket-create audit_manifest.json audit_ticket.json "Baseline certification ticket"
```

This writes `audit_ticket.json` containing the manifest, ticket signature, and summary.

## Verifying an Audit Ticket

Validate the audit ticket and repository contents with:

```bash
cargo run --bin audit_ticket_verifier -- verify-ticket audit_ticket.json
```

A successful verification prints a JSON report with `outcome: PASS`.

## Verifying a Manifest Only

If a reviewer wants to validate the manifest separately:

```bash
cargo run --bin audit_ticket_verifier -- verify-manifest audit_manifest.json
```

This command verifies the manifest hash and the repository file hashes.

## Expected Outcomes

- `PASS` — repository identity, manifest content, and artifact hashes match
- `PASS WITH CONDITIONS` — ticket is valid but git tag or repository identity is incomplete
- `FAIL` — ticket structure, signature, or artifact hashes are inconsistent

## Notes for Independent Reviewers

- The verifier is deterministic and requires no external secret keys.
- The signature is derived from the ticket contents and the certified baseline metadata.
- The local repository must be a Git checkout so that commit and tag metadata can be resolved.
