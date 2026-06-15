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

use crate::deterministic_core::DetTime;
use crate::failure_axis::{FailureAxis, SystemHalt};
use crate::interface_discovery::{DiscoveredEndpoint, ProtocolKind};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolTransactionTrace {
    pub timestamp_us: u64,
    pub device_id: String,
    pub protocol: ProtocolKind,
    pub direction: TransactionDirection,
    pub payload_hash_hex: String,
    pub binding_confirmed: bool,
    pub first_use: bool,
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTelemetry {
    pub protocol: ProtocolKind,
    pub summary: String,
}

pub trait ProtocolDriver {
    fn kind(&self) -> ProtocolKind;
    fn validate_endpoint(&self, endpoint: &DiscoveredEndpoint) -> bool;
    fn parse_telemetry(&self, payload: &[u8]) -> Result<ParsedTelemetry, SystemHalt>;
}

pub struct Dnp3Driver;
pub struct ModbusDriver;
pub struct Iec61850Driver;
pub struct C37p118Driver;
pub struct IccpTase2Driver;

impl ProtocolDriver for Dnp3Driver {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::DNP3
    }
    fn validate_endpoint(&self, endpoint: &DiscoveredEndpoint) -> bool {
        endpoint.port == 20000
    }
    fn parse_telemetry(&self, payload: &[u8]) -> Result<ParsedTelemetry, SystemHalt> {
        Ok(ParsedTelemetry {
            protocol: ProtocolKind::DNP3,
            summary: format!("DNP3 message: {} bytes", payload.len()),
        })
    }
}

impl ProtocolDriver for ModbusDriver {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::Modbus
    }
    fn validate_endpoint(&self, endpoint: &DiscoveredEndpoint) -> bool {
        endpoint.port == 502
    }
    fn parse_telemetry(&self, payload: &[u8]) -> Result<ParsedTelemetry, SystemHalt> {
        Ok(ParsedTelemetry {
            protocol: ProtocolKind::Modbus,
            summary: format!("Modbus message: {} bytes", payload.len()),
        })
    }
}

impl ProtocolDriver for Iec61850Driver {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::IEC61850
    }
    fn validate_endpoint(&self, endpoint: &DiscoveredEndpoint) -> bool {
        endpoint.port == 50000
    }
    fn parse_telemetry(&self, payload: &[u8]) -> Result<ParsedTelemetry, SystemHalt> {
        Ok(ParsedTelemetry {
            protocol: ProtocolKind::IEC61850,
            summary: format!("IEC-61850 message: {} bytes", payload.len()),
        })
    }
}

impl ProtocolDriver for C37p118Driver {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::C37p118
    }
    fn validate_endpoint(&self, endpoint: &DiscoveredEndpoint) -> bool {
        endpoint.port == 4712
    }
    fn parse_telemetry(&self, payload: &[u8]) -> Result<ParsedTelemetry, SystemHalt> {
        Ok(ParsedTelemetry {
            protocol: ProtocolKind::C37p118,
            summary: format!("C37.118 message: {} bytes", payload.len()),
        })
    }
}

impl ProtocolDriver for IccpTase2Driver {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::ICCP_TASE2
    }
    fn validate_endpoint(&self, endpoint: &DiscoveredEndpoint) -> bool {
        endpoint.port == 102
    }
    fn parse_telemetry(&self, payload: &[u8]) -> Result<ParsedTelemetry, SystemHalt> {
        Ok(ParsedTelemetry {
            protocol: ProtocolKind::ICCP_TASE2,
            summary: format!("ICCP-TASE2 message: {} bytes", payload.len()),
        })
    }
}

pub fn protocol_kind_by_port(port: u16) -> Option<ProtocolKind> {
    match port {
        20000 => Some(ProtocolKind::DNP3),
        502 => Some(ProtocolKind::Modbus),
        50000 => Some(ProtocolKind::IEC61850),
        4712 => Some(ProtocolKind::C37p118),
        102 => Some(ProtocolKind::ICCP_TASE2),
        _ => None,
    }
}

pub fn parse_payload_by_port(port: u16, payload: &[u8]) -> Result<ParsedTelemetry, SystemHalt> {
    match port {
        20000 => Dnp3Driver.parse_telemetry(payload),
        502 => ModbusDriver.parse_telemetry(payload),
        50000 => Iec61850Driver.parse_telemetry(payload),
        4712 => C37p118Driver.parse_telemetry(payload),
        102 => IccpTase2Driver.parse_telemetry(payload),
        _ => Err(SystemHalt::with_formatted(
            FailureAxis::ExternalInjectionDetected,
            &format!("Unsupported protocol port: {}", port),
        )),
    }
}

pub fn validate_discovered_protocols(endpoints: &[DiscoveredEndpoint]) -> Vec<(String, bool)> {
    endpoints
        .iter()
        .map(|ep| (ep.hostname.clone(), true))
        .collect()
}

#[derive(Debug, Default)]
pub struct ProtocolTraceSigner {
    traces: Vec<ProtocolTransactionTrace>,
}

impl ProtocolTraceSigner {
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
        }
    }

    pub fn sign_transaction(
        &mut self,
        device_id: String,
        protocol: ProtocolKind,
        direction: TransactionDirection,
        payload: &[u8],
    ) -> ProtocolTransactionTrace {
        let hash = sha256_hex(payload);
        let sig = sha256_hex(&[&hash.as_bytes(), device_id.as_bytes()].concat());

        ProtocolTransactionTrace {
            timestamp_us: DetTime::canonical_now_ms().as_millis() as u64,
            device_id,
            protocol,
            direction,
            payload_hash_hex: hash,
            binding_confirmed: true,
            first_use: self.traces.is_empty(),
            signature_hex: sig,
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dnp3_driver_validates_port() {
        let driver = Dnp3Driver;
        let ep = DiscoveredEndpoint {
            hostname: "test".to_string(),
            port: 20000,
            service: "dnp3".to_string(),
        };
        assert!(driver.validate_endpoint(&ep));
    }

    #[test]
    fn modbus_driver_validates_port() {
        let driver = ModbusDriver;
        let ep = DiscoveredEndpoint {
            hostname: "test".to_string(),
            port: 502,
            service: "modbus".to_string(),
        };
        assert!(driver.validate_endpoint(&ep));
    }
}
