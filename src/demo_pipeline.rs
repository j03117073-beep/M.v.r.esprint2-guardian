// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// TLBSS Demo Pipeline
// Unified demonstration of constraint evaluation → L7 mapping → adversarial validation

use serde::{Deserialize, Serialize};
use crate::{
    constraint_system::{ConstraintEvaluator, PowerState, Trajectory, ViolationVector},
    failure_axis::{FailureAxis, SystemHalt},
    fiel::execute_tick,
    testament_audit::TestamentAudit,
    tlbss_types::SubstrateNode,
};
use std::time::Instant;

/// L7 Event Types - Map to regulatory emergency actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Resource insufficiency → RUC / operator commit
    RUCRequired,
    /// Responsive reserves deployment
    ReserveDeploy,
    /// Scarcity pricing activation (ORDC)
    ScarcityPricing,
    /// Emergency transmission ratings
    EmergencyLimit,
    /// Load shedding (UFLS, last resort)
    LoadShed,
}

/// Market snapshot for replay scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub load_mw: f64,
    pub generation_mw: f64,
    pub reserve_margin_mw: f64,
    pub transmission_limits: Vec<f64>,
}

impl MarketSnapshot {
    pub fn normal() -> Self {
        Self {
            load_mw: 4500.0,
            generation_mw: 5200.0,
            reserve_margin_mw: 700.0,
            transmission_limits: vec![2000.0, 1800.0, 2200.0],
        }
    }

    pub fn reserve_shortage() -> Self {
        Self {
            load_mw: 4800.0,
            generation_mw: 4900.0,
            reserve_margin_mw: 100.0, // Insufficient reserves
            transmission_limits: vec![2000.0, 1800.0, 2200.0],
        }
    }

    pub fn capacity_shortage() -> Self {
        Self {
            load_mw: 5500.0,
            generation_mw: 4800.0, // Generation < Load
            reserve_margin_mw: -700.0,
            transmission_limits: vec![2000.0, 1800.0, 2200.0],
        }
    }

    pub fn network_overload() -> Self {
        Self {
            load_mw: 4600.0,
            generation_mw: 5100.0,
            reserve_margin_mw: 500.0,
            transmission_limits: vec![1800.0, 1600.0, 1900.0], // Overloaded lines
        }
    }

    pub fn collapse_case() -> Self {
        Self {
            load_mw: 6200.0,
            generation_mw: 4500.0,
            reserve_margin_mw: -1700.0,
            transmission_limits: vec![1500.0, 1400.0, 1600.0],
        }
    }

    pub fn stress_case() -> Self {
        Self::collapse_case()
    }
}

/// Complete demo result - shows full pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoResult {
    pub admissible: bool,
    pub violations: ViolationVector,
    pub l7_event: Option<EventType>,
    pub engine_halt: Option<FailureAxis>,
    pub audit_halt: Option<FailureAxis>,
    pub execution_time_ms: f64,
    pub trace_length: usize,
}

/// Convert market snapshot to trajectory for constraint evaluation
pub fn propose_trajectory_from_snapshot(snapshot: &MarketSnapshot) -> Trajectory {
    // Create a simple 3-interval trajectory based on market conditions
    let base_state = PowerState::new(
        snapshot.load_mw, // Current power demand
        snapshot.generation_mw * 0.75, // Previous power
        snapshot.reserve_margin_mw * 0.3, // Reg up
        snapshot.reserve_margin_mw * 0.2, // Reg down
        snapshot.generation_mw * 0.5, // Min power
        snapshot.generation_mw * 1.1, // Max power
        snapshot.generation_mw * 0.1, // Ramp up
        snapshot.generation_mw * 0.1, // Ramp down
    );

    // Create trajectory with slight variations
    let intervals = vec![
        base_state.clone(),
        PowerState::new(
            snapshot.generation_mw * 0.85,
            base_state.p_t,
            base_state.reg_up,
            base_state.reg_down,
            base_state.p_min,
            base_state.p_max,
            base_state.ramp_up,
            base_state.ramp_down,
        ),
        PowerState::new(
            snapshot.generation_mw * 0.9,
            base_state.p_t,
            base_state.reg_up,
            base_state.reg_down,
            base_state.p_min,
            base_state.p_max,
            base_state.ramp_up,
            base_state.ramp_down,
        ),
    ];

    Trajectory::new(intervals)
}

/// Evaluate trajectory for constraint violations
pub fn evaluate_trajectory(traj: &Trajectory) -> ViolationVector {
    ConstraintEvaluator::evaluate_trajectory(traj)
}

/// Map violations to L7 regulatory actions
pub fn map_violation_to_l7(v: &ViolationVector) -> Option<EventType> {
    // Critical thresholds for L7 activation
    const CRITICAL_THRESHOLD: f64 = 100.0;

    if v.capacity_upper > CRITICAL_THRESHOLD {
        return Some(EventType::RUCRequired);
    }

    if v.reg_up > CRITICAL_THRESHOLD / 2.0 {
        return Some(EventType::ReserveDeploy);
    }

    if v.ramp_up > CRITICAL_THRESHOLD / 3.0 {
        return Some(EventType::ReserveDeploy);
    }

    if v.capacity_lower > CRITICAL_THRESHOLD / 4.0 {
        return Some(EventType::EmergencyLimit);
    }

    if v.total() > CRITICAL_THRESHOLD * 2.0 {
        return Some(EventType::LoadShed);
    }

    None
}

/// Context-aware adversarial injection based on L7 event
pub fn inject_adversarial_sequence(
    event: Option<EventType>,
) -> (Vec<SubstrateNode>, Option<SystemHalt>) {
    match event {
        Some(EventType::RUCRequired) => reference_attack_sequence(),
        Some(EventType::ReserveDeploy) => feedback_attack_sequence(),
        Some(EventType::EmergencyLimit) => coupling_attack_sequence(),
        Some(EventType::LoadShed) => resolution_attack_sequence(),
        Some(EventType::ScarcityPricing) => mask_violation_sequence(),
        _ => authority_violation_sequence(),
    }
}

/// Clean execution for admissible cases
pub fn run_clean_execution() -> (Vec<SubstrateNode>, Option<SystemHalt>) {
    let mut trace = Vec::new();
    let mut node = SubstrateNode::new(0);

    // Normal operation trace
    for i in 0..10 {
        let signal = (0x5A ^ i as u8) as u8;
        if let Ok(()) = execute_tick(&mut node, i as u64, signal) {
            trace.push(node.clone());
        }
    }

    (trace, None)
}

/// MAIN DEMO PIPELINE - Shows complete system behavior
pub fn run_full_demo(snapshot: MarketSnapshot) -> DemoResult {
    let start = Instant::now();

    // Step 1: Replay - Convert market conditions to trajectory
    let traj = propose_trajectory_from_snapshot(&snapshot);

    // Step 2: Constraint Evaluation
    let violations = evaluate_trajectory(&traj);
    let admissible = violations.is_feasible();

    // Step 3: L6 → L7 Mapping
    let l7_event = if !admissible {
        map_violation_to_l7(&violations)
    } else {
        None
    };

    // Step 4: Execution with Adversarial Injection
    let (trace, engine_halt) = if !admissible {
        inject_adversarial_sequence(l7_event.clone())
    } else {
        run_clean_execution()
    };

    let trace_length = trace.len();

    // Step 5: Audit Evaluation
    let audit = TestamentAudit::new();
    let audit_halt = audit.evaluate(&trace);

    let execution_time_ms = start.elapsed().as_nanos() as f64 / 1_000_000.0;

    DemoResult {
        admissible,
        violations,
        l7_event,
        engine_halt: engine_halt.map(|h| h.axis),
        audit_halt: audit_halt.map(|h| h.axis),
        execution_time_ms,
        trace_length,
    }
}

/// Pretty print demo result for CLI
pub fn print_demo_pretty(result: &DemoResult) {
    println!("================ DEMO RESULT ================");
    println!("Admissible: {}", result.admissible);

    if !result.admissible {
        println!("⚠ Violations:");
        println!("  Capacity Upper: {:.1} MW", result.violations.capacity_upper);
        println!("  Capacity Lower: {:.1} MW", result.violations.capacity_lower);
        println!("  Ramp Up: {:.1} MW", result.violations.ramp_up);
        println!("  Ramp Down: {:.1} MW", result.violations.ramp_down);
        println!("  Reg Up: {:.1} MW", result.violations.reg_up);
        println!("  Reg Down: {:.1} MW", result.violations.reg_down);
        println!("  Total: {:.1} MW", result.violations.total());
    }

    match result.l7_event {
        Some(ref e) => println!("🚨 L7 Triggered: {:?}", e),
        None => println!("✅ No L7 Required"),
    }

    match result.engine_halt {
        Some(ref h) => println!("🛑 Engine Halt: {:?}", h),
        None => println!("Engine Stable"),
    }

    match result.audit_halt {
        Some(ref h) => println!("📋 Audit Halt: {:?}", h),
        None => println!("Audit Clean"),
    }

    println!("⏱️  Execution Time: {:.3} ms", result.execution_time_ms);
    println!("📊 Trace Length: {} nodes", result.trace_length);
    println!("=============================================");
}

// ============================================================================
// ATTACK SEQUENCES (Context-Aware Injection)
// ============================================================================

/// Reference corruption for capacity shortages
fn reference_attack_sequence() -> (Vec<SubstrateNode>, Option<SystemHalt>) {
    let mut trace = Vec::new();
    let mut node = SubstrateNode::new(0);

    for i in 0..10 {
        let signal = (0x5A ^ i as u8) as u8;
        if let Ok(()) = execute_tick(&mut node, i as u64, signal) {
            trace.push(node.clone());
        }
    }

    node.charge = node.charge.saturating_add(999_999);
    let halt = Some(SystemHalt::new(
        FailureAxis::Reference,
        "Reference counter corruption injected",
    ));

    trace.push(node);
    (trace, halt)
}

/// Feedback instability for reserve shortages
fn feedback_attack_sequence() -> (Vec<SubstrateNode>, Option<SystemHalt>) {
    let mut trace = Vec::new();
    let mut node = SubstrateNode::new(0);

    for i in 0..15 {
        let signal = 0x00;
        if let Ok(()) = execute_tick(&mut node, i as u64, signal) {
            trace.push(node.clone());
        }
    }

    node.stable_ticks = 255;
    let halt = Some(SystemHalt::new(
        FailureAxis::Feedback,
        "Feedback instability injected",
    ));

    trace.push(node);
    (trace, halt)
}

/// Coupling violation for network overloads
fn coupling_attack_sequence() -> (Vec<SubstrateNode>, Option<SystemHalt>) {
    let mut trace = Vec::new();
    let mut node = SubstrateNode::new(0);

    for i in 0..8 {
        let signal = (0xA5 | (i as u8 & 0x0F)) as u8;
        if let Ok(()) = execute_tick(&mut node, i as u64, signal) {
            trace.push(node.clone());
        }
    }

    node.entity_a = Some(Box::new(SubstrateNode::new(0xFF)));
    let halt = Some(SystemHalt::new(
        FailureAxis::Coupling,
        "Entity coupling violation injected",
    ));

    trace.push(node);
    (trace, halt)
}

/// Resolution overflow for collapse cases
fn resolution_attack_sequence() -> (Vec<SubstrateNode>, Option<SystemHalt>) {
    let mut trace = Vec::new();
    let mut node = SubstrateNode::new(0);

    for i in 0..20 {
        let signal = 0xFF;
        if let Ok(()) = execute_tick(&mut node, i as u64, signal) {
            trace.push(node.clone());
        }
    }

    node.charge = u64::MAX - 100;
    let halt = Some(SystemHalt::new(
        FailureAxis::Resolution,
        "Resolution overflow detected",
    ));

    trace.push(node);
    (trace, halt)
}

/// Mask violation for scarcity pricing
fn mask_violation_sequence() -> (Vec<SubstrateNode>, Option<SystemHalt>) {
    let mut trace = Vec::new();
    let mut node = SubstrateNode::new(0);

    for i in 0..5 {
        let signal = (i as u8) ^ 0x5A;
        if let Ok(()) = execute_tick(&mut node, i as u64, signal) {
            trace.push(node.clone());
        }
    }

    node.masked_signal = 0x7F;
    let halt = Some(SystemHalt::new(
        FailureAxis::InternalInvariantBreach,
        "Mask invariant violation injected",
    ));

    trace.push(node);
    (trace, halt)
}

/// Authority violation for emergency limits
fn authority_violation_sequence() -> (Vec<SubstrateNode>, Option<SystemHalt>) {
    let mut trace = Vec::new();
    let node = SubstrateNode::new(0);

    trace.push(node);

    let halt = Some(SystemHalt::new(
        FailureAxis::AuthorityInversionAttempt,
        "Authority escalation attempt blocked",
    ));

    (trace, halt)
}
