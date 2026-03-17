#![deny(unsafe_code)]

//! Core kernel enums and state machine for the deterministic supervisory gatekeeper.
//!
//! This module contains the authority levels the kernel may assert over incoming
//! setpoints as well as the internal fault-state machine that replaces any
//! previous `panic`/`exit` behaviour. The kernel does **not** itself compute
//! droop or primary control; it only validates and optionally clamps commands
//! issued by an upstream optimiser or AI layer.

/// How much authority the kernel is exercising over the current setpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAuthority {
    /// No intervention; the setpoint passes through unmodified.
    PassThrough,
    /// The command triggered a hard clamp (e.g. exceeding a physical limit).
    Clamp,
    /// The command was smoothed by a rate limiter.
    RateLimit,
    /// The kernel has given up and is delegating to native PPC droop mode.
    FallbackToDroop,
}

/// Operational state of the kernel.  Transitions are driven deterministically
/// by invariant violations and sensor/communication health.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelState {
    /// Normal operation; AI/PPC commands are being validated and forwarded.
    Normal,
    /// A mild violation occurred; outputs are being rate‑limited or clamped.
    Degraded,
    /// A major incoherence was detected; AI commands are ignored and the last
    /// valid setpoint is held.
    Incoherent,
    /// Emergency fallback: the kernel is no longer controlling setpoints and
    /// the plant is left to its internal droop/protection logic.
    Emergency,
}
