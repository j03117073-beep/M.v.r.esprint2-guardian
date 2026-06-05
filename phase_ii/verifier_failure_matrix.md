# Verifier Failure Matrix

## Objective
Prove `verifier` rejects malformed attestation evidence predictably.

## Artifacts
All test files are under `phase_ii/adversarial`.

| Case | Artifact | Expected Failure | Observed Result |
|---|---|---|---|
| 1 | `empty.json` | Parse failure on empty input | `Error: Error("EOF while parsing a value", line: 1, column: 0)` |
| 2 | `invalid_json.json` | Parse failure on invalid JSON | `Error: Error("invalid type: map, expected a sequence", line: 1, column: 0)` |
| 3 | `missing_signature.json` | Schema validation missing required field | `Error: Error("missing field `signature`", line: 106, column: 3)` |
| 4 | `missing_prev_hash.json` | Schema validation missing required field | `Error: Error("missing field `prev_hash`", line: 245, column: 3)` |
| 5 | `invalid_signature.json` | Signature verification failure | `Error: "Invalid signature"` |
| 6 | `invalid_prev_hash.json` | Linkage check failure | `Error: "Chain broken at index 1"` |
| 7 | `stale_timestamp.json` | Timestamp ordering check failure | `Error: "Timestamp ordering violated"` |
| 8 | `prefixed_invalid.json` | Parse failure on non-JSON prefix | `Error: Error("expected value", line: 1, column: 1)` |
| 9 | `truncated.json` | Parse failure on truncated JSON | `Error: Error("EOF while parsing a value", line: 215, column: 2)` |

## Conclusion
- `verifier` rejects malformed evidence consistently across parse, schema, signature, hash, and ordering failures.
- Failure modes are predictable and distinct.

## Status
- VERIFIED
