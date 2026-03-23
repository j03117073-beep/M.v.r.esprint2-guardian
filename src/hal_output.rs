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

use crate::failure_axis::SystemHalt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_PACKET_BYTES: usize = 32;

#[derive(Debug, Clone)]
pub struct OutputCommand {
    pub tick: u64,
    pub state_vector: [u8; 3],
    pub coherence_metric: f32,
    pub safe_state: bool,
}

impl OutputCommand {
    pub fn to_fixed_packet(&self) -> [u8; DEFAULT_PACKET_BYTES] {
        let mut packet = [0u8; DEFAULT_PACKET_BYTES];
        packet[..8].copy_from_slice(&self.tick.to_le_bytes());
        packet[8] = self.state_vector[0];
        packet[9] = self.state_vector[1];
        packet[10] = self.state_vector[2];
        packet[11] = u8::from(self.safe_state);
        packet
    }
}

#[derive(Debug, Clone)]
pub struct OutputHalPolicy {
    pub allowed_interfaces: Vec<String>,
    pub packet_bytes: usize,
    pub deadman_enabled: bool,
    pub keepalive_timeout_ms: u64,
    pub keepalive_file: Option<PathBuf>,
}

impl OutputHalPolicy {
    pub fn from_env() -> Self {
        Self {
            allowed_interfaces: vec!["eth0".to_string(), "sim".to_string()],
            packet_bytes: DEFAULT_PACKET_BYTES,
            deadman_enabled: true,
            keepalive_timeout_ms: 1000,
            keepalive_file: None,
        }
    }
}

pub struct DeterministicOutputHal {
    selected_interface: String,
    policy: OutputHalPolicy,
}

impl DeterministicOutputHal {
    pub fn from_env() -> Result<Self, SystemHalt> {
        let policy = OutputHalPolicy::from_env();
        Ok(Self {
            selected_interface: "sim".to_string(),
            policy,
        })
    }

    pub fn poll_keepalive(&self) -> Result<(), SystemHalt> {
        Ok(())
    }

    pub fn dispatch(&self, command: &OutputCommand) -> Result<(), SystemHalt> {
        let packet = command.to_fixed_packet();
        self.write_sim_packet(&packet)
    }

    pub fn enter_safe_state(&self, tick: u64) -> Result<(), SystemHalt> {
        let cmd = OutputCommand {
            tick,
            state_vector: [0, 0, 0],
            coherence_metric: 0.0,
            safe_state: true,
        };
        self.dispatch(&cmd)
    }

    fn write_sim_packet(&self, packet: &[u8; DEFAULT_PACKET_BYTES]) -> Result<(), SystemHalt> {
        // In simulation, just log to stderr
        eprintln!("[OUTPUT] {}", to_hex(packet));
        Ok(())
    }
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn now_us() -> Result<u64, SystemHalt> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_packet_layout_has_expected_size() {
        let cmd = OutputCommand {
            tick: 100,
            state_vector: [1, 2, 3],
            coherence_metric: 0.95,
            safe_state: false,
        };
        let packet = cmd.to_fixed_packet();
        assert_eq!(packet.len(), DEFAULT_PACKET_BYTES);
    }
}
