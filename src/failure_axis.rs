/* Failure axis registry for deterministic halts */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureAxis {
    InternalInvariantBreach,
    ExternalInjectionDetected,
    TimingDriftFailure,
    AuthorityInversionAttempt,

    // Additional axes introduced for audit/hardening
    Reference,
    Feedback,
    Coupling,
    Resolution,
    Axiom6_7Misalignment,
}

#[derive(Debug, Clone)]
pub struct SystemHalt {
    pub axis: FailureAxis,
    pub message: String,
}

impl SystemHalt {
    pub fn new(axis: FailureAxis, message: &str) -> Self {
        Self {
            axis,
            message: message.to_string(),
        }
    }

    pub fn with_formatted(axis: FailureAxis, message: String) -> Self {
        Self { axis, message }
    }
}
