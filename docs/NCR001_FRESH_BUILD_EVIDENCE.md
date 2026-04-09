# NCR-001 Fresh Build Evidence

Date: April 8, 2026  
Scope: clean release build evidence and binary hash capture for compliance/licensing bundle.

## Build Evidence

Command:
- `cargo build --release`

Result:
- `Finished release profile [optimized]` (workspace host).

Smoke validation:
- `target/release/sced_chain.exe verify test_vectors/gold_truth_sced_20260322_1805.csv` -> PASS

## SHA-256 Binary Hashes

- `sced_chain.exe`  
  `8b168ce7e761016fd5978733e8c2434878b1a36ec2725aa4f5b9d77f8115ffb0`

- `verifier.exe`  
  `9471a7967bef100e28691be3253a8ab0d2f5de0e0e22727c5c4922e4b438b9fd`

- `pilot_demo.exe`  
  `e3771d88c294a67b3aaf83f02251a30ba4bde3b0e8fca941199e1ed908900f97`

- `dashboard.exe`  
  `95229f3b76c9e4aa0994b63a72a9efcd1c0d10c05deab9b9abea44c02ac03701`

## Notes

- Hashes were generated using PowerShell `Get-FileHash -Algorithm SHA256`.
- This evidence can be reused for environment parity checks across Ubuntu/RHEL hosts.

