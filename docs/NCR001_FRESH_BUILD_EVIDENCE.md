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

## April 11, 2026 Onboarding Run

Source:
- `ercot_onboard.sh` (WSL Ubuntu run)

Output:
- `READY_FOR_ERCOT_REVIEW.txt`

Summary:
- `ri04=PASS`
- `ri12=PASS`
- `ri18=PASS`
- `sha256=06a32540ccbb5d3754e9883422a3355c7435d3326ef2f2c69f6d74918a2a4001`
