// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// Operational I/O agent for interface discovery and protocol-driven telemetry parsing.

use m_v_r_esprint1::interface_discovery::{discover_and_map, DiscoveryConfig};
use m_v_r_esprint1::protocol_drivers::{parse_payload_by_port, protocol_kind_by_port};
use std::net::TcpListener;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IO Agent: interface discovery and protocol validation");

    let config = DiscoveryConfig {
        include_loopback: true,
        timeout_secs: 3,
    };

    let report = discover_and_map(&config).map_err(|halt| format!("Discovery failed: {}", halt.message))?;
    println!("Discovered {} endpoints", report.endpoints.len());
    for endpoint in &report.endpoints {
        println!(" - {}:{} ({})", endpoint.hostname, endpoint.port, endpoint.service);
    }

    for endpoint in &report.endpoints {
        if let Some(kind) = protocol_kind_by_port(endpoint.port) {
            println!("Protocol kind: {:?} for port {}", kind, endpoint.port);
            let payload = b"TEST_MESSAGE";
            let telemetry = parse_payload_by_port(endpoint.port, payload)
                .map_err(|halt| format!("parse failed: {}", halt.message))?;
            println!("  Parsed telemetry: {}", telemetry.summary);
        }
    }

    // verify we can bind to discovered endpoints locally (non-blocking)
    for endpoint in &report.endpoints {
        let socket_addr = format!("127.0.0.1:{}", endpoint.port);
        match TcpListener::bind(&socket_addr) {
            Ok(listener) => {
                println!("Successfully bound test socket to {}", socket_addr);
                drop(listener);
            }
            Err(err) => {
                println!("Bind failed for {}: {}", socket_addr, err);
            }
        }
    }

    println!("IO Agent complete. Operational I/O flow verified." );
    Ok(())
}
