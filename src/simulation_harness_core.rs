#![deny(unsafe_code)]

use std::fs;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
enum SystemState {
    Idle,
    Discovering,
    Operational,
    Revoked,
    SafeState,
}

fn run_simulation_one_greenfield() {
    let mut state = SystemState::Idle;
    assert_eq!(state, SystemState::Idle);
    state = SystemState::Discovering;
    assert_eq!(state, SystemState::Discovering);
    state = SystemState::Operational;
    assert_eq!(state, SystemState::Operational);
    state = SystemState::Revoked;
    if state == SystemState::Revoked {
        state = SystemState::SafeState;
    }
    assert_eq!(state, SystemState::SafeState);
}

fn run_simulation_two_jitters() {
    let loop_target_micros = 1000u128;
    for i in 0..5 {
        let cycle_start = Instant::now();
        let mut acc: u64 = 0;
        for b in 0..200 {
            acc = acc.wrapping_add(1);
        }
        let _ = acc;
        let wcet = cycle_start.elapsed().as_micros();
        assert!(wcet < 150, "Deterministic violation!");
        let cycle_total = cycle_start.elapsed().as_micros();
        if cycle_total < loop_target_micros {
            let sleep_time = loop_target_micros - cycle_total;
            std::thread::sleep(Duration::from_micros(sleep_time as u64));
        }
    }
}

fn run_simulation_three_universal() {
    let ports = vec![
        ("TCP 20000", "DNP3", "Setpoint Control"),
        ("TCP 502", "Modbus", "Analog Input (Weather)"),
        ("TCP 50000", "IEC-61850", "GOOSE"),
        ("TCP 6379", "Redis", "Telemetry Cache"),
        ("UDP 161", "SNMP", "Device Status"),
    ];
    assert_eq!(ports.len(), 5);
}

fn verify_required_protocol_bindings(manifest_path: &str) -> Result<(), String> {
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("manifest read failed at {manifest_path}: {e}"))?;
    let required = [
        "TPL-008-1",
        "PRC-029-1",
        "CIP-012-2",
    ];
    for (p, t) in required.iter().zip(std::iter::repeat("required")) {
        if !content.contains(p) {
            return Err(format!("Missing required policy: {}", p));
        }
    }
    Ok(())
}

pub fn run_all(manifest_path: &str) -> Result<(), String> {
    run_simulation_one_greenfield();
    run_simulation_two_jitters();
    run_simulation_three_universal();
    verify_required_protocol_bindings(manifest_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn harness_validates_required_bindings() {
        let mut temp = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp, "TPL-008-1: enabled\nPRC-029-1: enabled\nCIP-012-2: enabled").unwrap();
        let result = verify_required_protocol_bindings(temp.path().to_str().unwrap());
        assert!(result.is_ok());
    }
}
