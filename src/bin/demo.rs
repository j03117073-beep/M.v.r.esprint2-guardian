// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// TLBSS Demo CLI Tool
// Command-line interface for running the complete demo pipeline

use m_v_r_esprint1::demo_pipeline::{run_full_demo, print_demo_pretty, MarketSnapshot};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("🚀 TLBSS Demo Pipeline");
        println!("Usage: {} <scenario>", args[0]);
        println!();
        println!("Available scenarios:");
        println!("  normal      - Normal operations");
        println!("  reserve     - Reserve shortage");
        println!("  capacity    - Capacity shortage");
        println!("  network     - Network overload");
        println!("  collapse    - System collapse");
        println!("  all         - Run all scenarios");
        return;
    }

    let scenario = &args[1];

    match scenario.as_str() {
        "normal" => run_single_scenario("Normal Operation", MarketSnapshot::normal()),
        "reserve" => run_single_scenario("Reserve Shortage", MarketSnapshot::reserve_shortage()),
        "capacity" => run_single_scenario("Capacity Shortage", MarketSnapshot::capacity_shortage()),
        "network" => run_single_scenario("Network Overload", MarketSnapshot::network_overload()),
        "collapse" => run_single_scenario("System Collapse", MarketSnapshot::collapse_case()),
        "all" => run_all_scenarios(),
        _ => {
            println!("❌ Unknown scenario: {}", scenario);
            println!("Run with no arguments to see available scenarios.");
        }
    }
}

fn run_single_scenario(name: &str, snapshot: MarketSnapshot) {
    println!("🚀 TLBSS DEMO PIPELINE");
    println!("Scenario: {}", name);
    println!();

    let result = run_full_demo(snapshot);
    print_demo_pretty(&result);
}

fn run_all_scenarios() {
    println!("🚀 TLBSS DEMO PIPELINE - ALL SCENARIOS");
    println!();

    let scenarios = vec![
        ("Normal Operation", MarketSnapshot::normal()),
        ("Reserve Shortage", MarketSnapshot::reserve_shortage()),
        ("Capacity Shortage", MarketSnapshot::capacity_shortage()),
        ("Network Overload", MarketSnapshot::network_overload()),
        ("System Collapse", MarketSnapshot::collapse_case()),
    ];

    for (name, snapshot) in scenarios {
        println!("=== {} ===", name);
        let result = run_full_demo(snapshot);
        print_demo_pretty(&result);
        println!();
    }
}
