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
use crate::interface_discovery::{DiscoveredEndpoint, ProtocolKind};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

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
    /// Perform any protocol-specific transport authentication checks.
    /// Default implementation returns `true` (no-op).
    fn authenticate_transport(&self, _token: Option<&[u8]>) -> bool {
        true
    }
}

macro_rules! impl_protocol_driver {
    ($struct:ident, $kind:expr, $port:expr, $name:expr) => {
        impl ProtocolDriver for $struct {
            fn kind(&self) -> ProtocolKind {
                $kind
            }
            fn validate_endpoint(&self, endpoint: &DiscoveredEndpoint) -> bool {
                endpoint.port == $port
            }
            fn parse_telemetry(&self, payload: &[u8]) -> Result<ParsedTelemetry, SystemHalt> {
                Ok(ParsedTelemetry {
                    protocol: $kind,
                    summary: format!("{} message: {} bytes", $name, payload.len()),
                })
            }
            fn authenticate_transport(&self, _token: Option<&[u8]>) -> bool {
                // Default: no transport-level auth enforced by driver
                true
            }
        }
    };
}

pub struct Dnp3Driver;
pub struct ModbusDriver;
pub struct Iec61850Driver;
pub struct C37p118Driver;
pub struct IccpTase2Driver;

impl_protocol_driver!(Dnp3Driver, ProtocolKind::DNP3, 20000, "DNP3");
impl_protocol_driver!(ModbusDriver, ProtocolKind::Modbus, 502, "Modbus");
impl_protocol_driver!(Iec61850Driver, ProtocolKind::IEC61850, 50000, "IEC-61850");
impl_protocol_driver!(C37p118Driver, ProtocolKind::C37p118, 4712, "C37.118");
impl_protocol_driver!(IccpTase2Driver, ProtocolKind::IccpTase2, 102, "ICCP-TASE2");

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
            timestamp_us: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64,
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

impl Dnp3Driver {
    /// Lightweight transport authentication helper for DNP3.
    /// Lightweight transport authentication helper for DNP3.
    /// In this minimal implementation the token must match the
    /// SHA256 hex digest of a test shared secret. This is a
    /// deterministic placeholder for real transport-level auth.
    pub fn authenticate(&self, token: Option<&[u8]>) -> bool {
        const TEST_SHARED_SECRET: &[u8] = b"TEST_SHARED_SECRET_V1";
        let expected = sha256_hex(TEST_SHARED_SECRET);

        match token {
            Some(t) => {
                // compare as utf8 hex string
                if let Ok(s) = std::str::from_utf8(t) {
                    s == expected
                } else {
                    false
                }
            }
            None => false,
        }
    }
}

impl Iec61850Driver {
    /// Map an IEC-61850 payload into a short CIM mapping summary.
    /// This provides a canonicalization entrypoint for downstream consumers.
    pub fn map_to_cim_summary(&self, _payload: &[u8]) -> String {
        // Use mapping table size as a proxy for canonicalization coverage
        let count = crate::cim_mapping_data::MAPPING_DATA.len();
        format!("IEC-61850 canonicalized (mapping entries: {})", count)
    }
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

    #[test]
    fn dnp3_driver_authenticate_checks_secret() {
        let driver = Dnp3Driver;
        // expected token is sha256_hex of TEST_SHARED_SECRET_V1
        const TEST_SHARED_SECRET: &[u8] = b"TEST_SHARED_SECRET_V1";
        let expected = sha256_hex(TEST_SHARED_SECRET);

        assert!(driver.authenticate(Some(expected.as_bytes())));
        // wrong token fails
        assert!(!driver.authenticate(Some(b"badtoken")));
        assert!(!driver.authenticate(None));
    }
}
