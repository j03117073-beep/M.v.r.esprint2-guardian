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

/// L7 Transition System - Market Emergency Actions Mapping
///
/// This module implements the broadcast mechanism for L7 transitions,
/// which map directly to ERCOT/PJM regulatory emergency actions:
///
/// - ExternalTransitionSignal → Operator intervention / emergency action
/// - Reason codes map to specific regulatory mechanisms:
///   * 0x0001: RUC / operator commit (resource insufficiency)
///   * 0x0002: Reserve deployment (responsive reserves)
///   * 0x0003: Scarcity pricing activation (ORDC)
///   * 0x0004: Emergency transmission ratings
///   * 0x0005: Load shedding (UFLS, last resort)
///
/// All transitions are logged, time-limited, and externally approved
/// per regulatory requirements. The kernel never acts directly.

use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

use crate::failure_axis::{FailureAxis, SystemHalt};

const DEFAULT_MULTICAST_IP: Ipv4Addr = Ipv4Addr::new(239, 10, 10, 10);
const DEFAULT_MULTICAST_PORT: u16 = 5001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionOpcode {
    ExternalTransitionSignal = 0xE7,
}

/// Zero-wait broadcast path for synchronized L7 transition signalling.
pub struct TransitionBroadcaster {
    socket: UdpSocket,
    target: SocketAddrV4,
}

impl TransitionBroadcaster {
    pub fn default_multicast() -> Result<Self, SystemHalt> {
        Self::new(DEFAULT_MULTICAST_IP, DEFAULT_MULTICAST_PORT)
    }

    pub fn new(multicast_ip: Ipv4Addr, port: u16) -> Result<Self, SystemHalt> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).map_err(|e| {
            SystemHalt::with_formatted(
                FailureAxis::ExternalInjectionDetected,
                format!("Transition UDP bind failed: {e}"),
            )
        })?;
        socket.set_nonblocking(true).map_err(|e| {
            SystemHalt::with_formatted(
                FailureAxis::TimingDriftFailure,
                format!("Transition UDP nonblocking setup failed: {e}"),
            )
        })?;
        socket
            .set_multicast_ttl_v4(1)
            .and_then(|_| socket.set_multicast_loop_v4(false))
            .map_err(|e| {
                SystemHalt::with_formatted(
                    FailureAxis::TimingDriftFailure,
                    format!("Transition UDP multicast setup failed: {e}"),
                )
            })?;
        Ok(Self {
            socket,
            target: SocketAddrV4::new(multicast_ip, port),
        })
    }

    pub fn broadcast_external_transition_signal(
        &self,
        tick: u64,
        reason_code: u16,
    ) -> Result<usize, SystemHalt> {
        let mut packet = [0u8; 16];
        packet[0] = TransitionOpcode::ExternalTransitionSignal as u8;
        packet[1] = 1; // protocol version
        packet[2..4].copy_from_slice(&reason_code.to_le_bytes());
        packet[4..12].copy_from_slice(&tick.to_le_bytes());
        let crc = packet[..12]
            .iter()
            .fold(0u32, |acc, b| acc.wrapping_add(*b as u32));
        packet[12..16].copy_from_slice(&crc.to_le_bytes());

        self.socket.send_to(&packet, self.target).map_err(|e| {
            SystemHalt::with_formatted(
                FailureAxis::TimingDriftFailure,
                format!("Transition multicast send failed: {e}"),
            )
        })
    }
}
