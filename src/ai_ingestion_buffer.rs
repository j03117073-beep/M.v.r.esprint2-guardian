// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// This file is part of the M.V.R.ESPRINT1 Sovereign Execution System,
// including TLBSS geometry, the Universal Execution Layer, the
// Deterministic IR, Rust Codegen Pipeline, SovereignBus, and the
// Cryptographic Audit Chain.
//
// No part of this file, its algorithms, structures, or designs may be
// copied, reproduced, modified, distributed, published, sublicensed,
// reverse-engineered, or used to create derivative works without the
// express written permission of OBINNA JAMES EJIOFOR.
//
// This software contains proprietary trade secrets and confidential
// intellectual property. Unauthorized use is strictly prohibited.


#![deny(unsafe_code)]

//! A very simple bounded buffer used to shuttle AI/PPC setpoints into the
//! deterministic 1 kHz kernel loop.  The implementation leverages the
//! `std::sync::mpsc::sync_channel` API and only uses the **non‑blocking**
//! `try_send`/`try_recv` methods in the real‑time path, guaranteeing that the
//! loop never blocks.  The buffer size is fixed at creation and is analogous to
//! a lock‑free ring buffer from the perspective of the consumer.

use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};

use crate::setpoint_guard::Setpoint;

pub struct AiIngestionBuffer {
    tx: SyncSender<Setpoint>,
    rx: Receiver<Setpoint>,
}

impl AiIngestionBuffer {
    /// Return a sender handle that can be moved to another thread.  The
    /// returned value is a lightweight clone of the underlying `SyncSender`.
    pub fn sender_clone(&self) -> SyncSender<Setpoint> {
        self.tx.clone()
    }
}

impl AiIngestionBuffer {
    /// Create a new buffer with the given capacity.  A power‑of‑two size is
    /// recommended for spectral reasons, but not required by the implementation.
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = sync_channel(capacity);
        AiIngestionBuffer { tx, rx }
    }

    /// Attempt to insert a setpoint.  If the buffer is full, the value is
    /// returned to the caller (AI layer) so it can be retried or discarded.
    pub fn push(&self, sp: Setpoint) -> Result<(), Setpoint> {
        match self.tx.try_send(sp) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(s)) => Err(s),
            Err(TrySendError::Disconnected(s)) => Err(s),
        }
    }

    /// Attempt to retrieve the next setpoint.  Returns `None` if the buffer is
    /// empty; this allows the real‑time loop to proceed without delay.
    pub fn pop(&self) -> Option<Setpoint> {
        match self.rx.try_recv() {
            Ok(sp) => Some(sp),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }
}
