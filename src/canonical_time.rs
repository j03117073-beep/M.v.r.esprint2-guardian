// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// CanonicalTime provides a deterministic timestamp representation for
// execution input and trace artifacts. It must be injected from the runtime
// gateway layer and never derived from host wall clock or instant APIs inside
// the core deterministic execution boundary.

#![deny(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalTime(pub u64);

impl CanonicalTime {
    pub fn from_millis(value: u64) -> Self {
        CanonicalTime(value)
    }
}
