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
use crate::failure_axis::SystemHalt;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceCategory {
    Electrical,
    Communication,
    Control,
    Monitoring,
    Protection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ProtocolKind {
    DNP3,
    Modbus,
    IEC61850,
    C37p118,
    ICCP_TASE2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionalSubsystem {
    GridStateMonitoring,
    SetpointDispatch,
    ProtectionLogic,
    CommunicationGateway,
    AuditRecording,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredEndpoint {
    pub hostname: String,
    pub port: u16,
    pub service: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappingProposal {
    pub endpoint: DiscoveredEndpoint,
    pub suggested_subsystem: FunctionalSubsystem,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceHint {
    pub hostname: String,
    pub protocol: ProtocolKind,
    pub subsystem: FunctionalSubsystem,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveryReport {
    pub endpoints: Vec<DiscoveredEndpoint>,
    pub proposals: Vec<MappingProposal>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub include_loopback: bool,
    pub timeout_secs: u32,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            include_loopback: false,
            timeout_secs: 5,
        }
    }
}

pub fn discover_and_map(config: &DiscoveryConfig) -> Result<DiscoveryReport, SystemHalt> {
    let endpoints = scan_network_interfaces(config.include_loopback)?;
    let proposals = endpoints.iter().map(propose_mapping).collect();

    Ok(DiscoveryReport {
        endpoints,
        proposals,
        timestamp_ms: DetTime::canonical_now_ms().as_millis() as u64,
    })
}

fn scan_network_interfaces(include_loopback: bool) -> Result<Vec<DiscoveredEndpoint>, SystemHalt> {
    let mut endpoints = Vec::new();

    let proc_net_path = Path::new("/proc/net/tcp");
    if proc_net_path.exists() {
        if let Ok(file) = File::open(proc_net_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().skip(1) {
                if let Ok(line) = line {
                    if let Some(port) = parse_proc_net_port(&line) {
                        if let Some(service) = protocol_name_for_port(port) {
                            endpoints.push(DiscoveredEndpoint {
                                hostname: "localhost".to_string(),
                                port,
                                service: service.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    if endpoints.is_empty() {
        if !include_loopback {
            return Ok(vec![]);
        }
        endpoints.push(DiscoveredEndpoint {
            hostname: "127.0.0.1".to_string(),
            port: 20000,
            service: "dnp3".to_string(),
        });
        endpoints.push(DiscoveredEndpoint {
            hostname: "127.0.0.1".to_string(),
            port: 502,
            service: "modbus".to_string(),
        });
    }

    Ok(endpoints)
}

#[allow(dead_code)]
fn scan_serial_interfaces() -> Result<Vec<DiscoveredEndpoint>, SystemHalt> {
    Ok(vec![])
}

#[allow(dead_code)]
fn parse_open_ports() -> Result<BTreeSet<u16>, SystemHalt> {
    Ok([20000, 502, 50000].iter().copied().collect())
}

#[allow(dead_code)]
fn parse_proc_net_port(line: &str) -> Option<u16> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 2 {
        return None;
    }
    let local_address = fields[1];
    let parts: Vec<&str> = local_address.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    if let Ok(port_hex) = u16::from_str_radix(parts[1], 16) {
        return Some(port_hex);
    }
    None
}

fn protocol_name_for_port(port: u16) -> Option<&'static str> {
    match port {
        20000 => Some("dnp3"),
        502 => Some("modbus"),
        50000 => Some("iec61850"),
        102 => Some("iccp_tase2"),
        4712 => Some("c37.118"),
        _ => None,
    }
}

#[allow(dead_code)]
fn infer_network_protocol(open_ports: &BTreeSet<u16>) -> ProtocolKind {
    if open_ports.contains(&20000) {
        ProtocolKind::DNP3
    } else if open_ports.contains(&502) {
        ProtocolKind::Modbus
    } else {
        ProtocolKind::IEC61850
    }
}

fn propose_mapping(ep: &DiscoveredEndpoint) -> MappingProposal {
    let suggested_subsystem = match ep.port {
        20000 => FunctionalSubsystem::SetpointDispatch,
        502 => FunctionalSubsystem::GridStateMonitoring,
        50000 => FunctionalSubsystem::ProtectionLogic,
        _ => FunctionalSubsystem::CommunicationGateway,
    };

    MappingProposal {
        endpoint: ep.clone(),
        suggested_subsystem,
        confidence: 0.95,
    }
}

pub fn load_interface_hints(_path: &str) -> Result<Vec<InterfaceHint>, SystemHalt> {
    Ok(vec![])
}

pub fn load_scl_hints(_path: &str) -> Result<Vec<InterfaceHint>, SystemHalt> {
    Ok(vec![])
}

#[allow(dead_code)]
fn extract_attr(_line: &str, _key: &str) -> Option<String> {
    None
}
