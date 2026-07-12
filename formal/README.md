# Isabelle formalization for the kernel

This directory contains a minimal Isabelle/HOL session for the kernel state machine modeled in [src/kernel.rs](../src/kernel.rs).

## Files
- `Kernel_Verification.thy`: core formalization of the kernel authority/state transitions and safety lemmas.
- `ROOT`: Isabelle session entrypoint for batch builds.

## Build
If Isabelle is installed locally, run:

```bash
isabelle build -d formal VerifiedKernel
```

## Purpose
The formalization captures the intended safety rule that a kernel in a normal or degraded state should not enter the emergency state unless the authority explicitly requests a fallback or the state is already incoherent.
