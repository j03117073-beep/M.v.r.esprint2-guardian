// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// This file is part of the M.V.R.ESPRINT1 Sovereign Execution System,
// including TLBSS geometry, the Universal Execution Layer, the
// Deterministic IR, Rust Codegen Pipeline, SovereignBus, and the
// Cryptographic Audit Chain.

use m_v_r_esprint1::audit_ticket::{create_audit_ticket, generate_manifest, repository_info, verify_ticket, AuditTicket, Manifest};
use serde_json;
use std::env;
use std::fs;

fn usage() {
    println!("Usage:");
    println!("  audit_ticket_verifier manifest-generate <output.json>");
    println!("  audit_ticket_verifier ticket-create <manifest.json> <output.json> <summary>");
    println!("  audit_ticket_verifier verify-ticket <ticket.json>");
    println!("  audit_ticket_verifier verify-manifest <manifest.json>");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let cwd = env::current_dir()?;
    let repo_root = cwd.as_path();

    match command.as_str() {
        "manifest-generate" => {
            let output = args.next().ok_or("output path is required")?;
            let repo_info = repository_info(repo_root)?;
            let manifest = generate_manifest(repo_root, repo_info.commit.clone(), repo_info.tag.unwrap_or_default())?;
            let json = serde_json::to_string_pretty(&manifest)?;
            fs::write(output, json)?;
            println!("✔ Audit manifest written");
        }
        "ticket-create" => {
            let manifest_path = args.next().ok_or("manifest path is required")?;
            let output = args.next().ok_or("output path is required")?;
            let summary = args.next().ok_or("summary is required")?;
            let content = fs::read_to_string(manifest_path)?;
            let manifest: Manifest = serde_json::from_str(&content)?;
            let ticket = create_audit_ticket(&manifest, summary)?;
            let json = serde_json::to_string_pretty(&ticket)?;
            fs::write(output, json)?;
            println!("✔ Audit ticket written");
        }
        "verify-ticket" => {
            let ticket_path = args.next().ok_or("ticket path is required")?;
            let content = fs::read_to_string(ticket_path)?;
            let ticket: AuditTicket = serde_json::from_str(&content)?;
            let report = verify_ticket(&ticket, repo_root);
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
            if report.outcome == "FAIL" {
                std::process::exit(1);
            }
        }
        "verify-manifest" => {
            let manifest_path = args.next().ok_or("manifest path is required")?;
            let content = fs::read_to_string(manifest_path)?;
            let manifest: Manifest = serde_json::from_str(&content)?;
            let reconciled_hash = manifest.manifest_hash.clone();
            let computed_hash = m_v_r_esprint1::audit_ticket::compute_manifest_hash(&manifest)?;
            if reconciled_hash != computed_hash {
                println!("FAIL: manifest hash mismatch\nexpected={}\nactual={}", reconciled_hash, computed_hash);
                std::process::exit(1);
            }
            let (missing, mismatches) = m_v_r_esprint1::audit_ticket::verify_manifest_entries(repo_root, &manifest);
            if !missing.is_empty() || !mismatches.is_empty() {
                println!("FAIL: manifest verification produced mismatches");
                for missing_file in missing {
                    println!("missing: {}", missing_file);
                }
                for mismatch in mismatches {
                    println!("mismatch: {}", mismatch);
                }
                std::process::exit(1);
            }
            println!("PASS: manifest verified");
        }
        _ => {
            usage();
            std::process::exit(1);
        }
    }

    Ok(())
}
