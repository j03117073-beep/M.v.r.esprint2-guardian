// Copyright (c) 2026 OBINNA JAMES EJIOFOR
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

use m_v_r_esprint1::sovereign_kernel::{
    attestation_record_data, build_signature_payload, has_identity_binding, hash_record, record_time,
    sign_payload_for_actor, verify_command_envelope, AttestationRecord,
};
use m_v_r_esprint1::tpm_attestation::{build_attestation_nonce, verify_quote_nonce_and_signature};
use m_v_r_esprint1::trusted_time::TrustedTime;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("provide log file path");

    let data = fs::read_to_string(&path)?;

    let records: Vec<AttestationRecord> = serde_json::from_str(&data)?;

    verify_chain(&records)?;

    if let Some(ticket) = build_audit_chain_ticket(&records) {
        write_ticket_artifacts(&ticket)?;
        println!("Audit ticket: {}", ticket.ticket_id);
        println!("Barcode payload: {}", ticket.barcode_payload);
        if let Some(snapshot) = &ticket.market_snapshot {
            println!(
                "Market snapshot hash ({} files): {}",
                snapshot.file_count, snapshot.snapshot_hash_hex
            );
        }
        println!("Barcode SVG: artifacts/audit_chain_ticket_barcode.svg");
    }

    println!("Chain verified: {} records valid", records.len());

    Ok(())
}

fn verify_chain(records: &[AttestationRecord]) -> Result<(), String> {
    for i in 0..records.len() {
        let record = &records[i];
        let current_time = record_time(record);

        // 1. Verify legacy attestation signature.
        verify_signature(record, &current_time)?;

        // 2. Verify hash linkage.
        if i > 0 {
            let prev = &records[i - 1];
            let prev_time = record_time(prev);
            let prev_data = attestation_record_data(prev);
            let expected = if !prev.record_hash.is_empty() {
                prev.record_hash.clone()
            } else {
                hash_record(&prev.prev_hash, &prev_data, &prev_time)
            };

            if expected != record.prev_hash {
                return Err(format!("Chain broken at index {}", i));
            }
        } else if record.prev_hash != vec![0; 32] {
            // First record should have prev_hash as zeros.
            return Err("First record prev_hash not zero".into());
        }

        // 3. Monotonic trusted-time check.
        if i > 0 {
            let prev_time = record_time(&records[i - 1]);
            verify_time_sequence(&prev_time, &current_time)?;
        }

        // 4. Identity, RBAC, and non-repudiation checks when identity binding exists.
        if has_identity_binding(record) {
            verify_command_envelope(record)?;
        }

        // 5. Recompute record hash and verify nonce/quote binding when attestation fields exist.
        if !record.record_hash.is_empty() {
            let record_data = attestation_record_data(record);
            let expected_record_hash = hash_record(&record.prev_hash, &record_data, &current_time);
            if expected_record_hash != record.record_hash {
                return Err("ERR_RECORD_HASH_MISMATCH".to_string());
            }

            let nonce = build_attestation_nonce(&record.decision_hash, &record.record_hash, &current_time);
            verify_quote_nonce_and_signature(&record.tpm_attest, &record.tpm_signature, &nonce)?;
        }
    }

    Ok(())
}

fn verify_time_sequence(prev: &TrustedTime, curr: &TrustedTime) -> Result<(), String> {
    if curr.monotonic_ns <= prev.monotonic_ns {
        return Err("ERR_TIME_REVERSAL".to_string());
    }

    if curr.source != prev.source {
        eprintln!("WARN: Time source changed");
    }

    Ok(())
}

fn verify_signature(record: &AttestationRecord, time: &TrustedTime) -> Result<(), String> {
    let expected_payload = build_signature_payload(&record.decision_hash, &record.pcr_digest, time);

    // Legacy simulation records used unsigned payloads directly.
    if expected_payload == record.signature {
        return Ok(());
    }

    // Current simulated signer mode signs payload with the fixed simulation key.
    let mut hasher = Sha256::new();
    hasher.update(&expected_payload);
    hasher.update(b"simulated-key");
    let expected_sim_signature = hasher.finalize().to_vec();

    if expected_sim_signature == record.signature {
        return Ok(());
    }

    Err("Invalid signature".into())
}

#[derive(Debug, Clone)]
struct AuditChainTicket {
    ticket_id: String,
    anchor_hash_hex: String,
    barcode_payload: String,
    market_snapshot: Option<MarketSnapshot>,
}

#[derive(Debug, Clone)]
struct MarketSnapshot {
    source_path: String,
    file_count: usize,
    snapshot_hash_hex: String,
    snapshot_signature_hex: Option<String>,
}

fn build_audit_chain_ticket(records: &[AttestationRecord]) -> Option<AuditChainTicket> {
    let latest_record = records.last()?;
    let anchor_bytes = records.last().map(|r| {
        if r.record_hash.is_empty() {
            &r.decision_hash
        } else {
            &r.record_hash
        }
    })?;

    let anchor_hash_hex = hex::encode(anchor_bytes);
    let short = &anchor_hash_hex[..anchor_hash_hex.len().min(12)];
    let ticket_id = format!("AUDIT-{}-{}", records.len(), short.to_uppercase());
    let market_snapshot = hash_market_conditions_snapshot(
        "Grid and Market Conditions",
        &latest_record.actor,
        &latest_record.command.command_id,
    );

    let barcode_payload = if let Some(snapshot) = &market_snapshot {
        format!(
            "MVR-AUDIT|{}|{}|MC:{}",
            ticket_id, anchor_hash_hex, snapshot.snapshot_hash_hex
        )
    } else {
        format!("MVR-AUDIT|{}|{}", ticket_id, anchor_hash_hex)
    };

    Some(AuditChainTicket {
        ticket_id,
        anchor_hash_hex,
        barcode_payload,
        market_snapshot,
    })
}

fn write_ticket_artifacts(ticket: &AuditChainTicket) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("artifacts")?;

    let mut txt = format!(
        "ticket_id={}\nanchor_hash={}\nbarcode_payload={}\nascii_barcode={}\n",
        ticket.ticket_id,
        ticket.anchor_hash_hex,
        ticket.barcode_payload,
        render_ascii_barcode(&ticket.barcode_payload),
    );
    if let Some(snapshot) = &ticket.market_snapshot {
        txt.push_str(&format!(
            "market_snapshot_source={}\nmarket_snapshot_file_count={}\nmarket_snapshot_hash={}\n",
            snapshot.source_path, snapshot.file_count, snapshot.snapshot_hash_hex
        ));
        if let Some(sig) = &snapshot.snapshot_signature_hex {
            txt.push_str(&format!("market_snapshot_signature={}\n", sig));
        }
    }
    fs::write("artifacts/audit_chain_ticket.txt", txt)?;

    let svg = render_barcode_svg(&ticket.barcode_payload);
    fs::write("artifacts/audit_chain_ticket_barcode.svg", svg)?;

    Ok(())
}

fn hash_market_conditions_snapshot(
    folder: &str,
    actor: &m_v_r_esprint1::sovereign_kernel::ActorContext,
    command_id: &str,
) -> Option<MarketSnapshot> {
    let path = Path::new(folder);
    if !path.exists() || !path.is_dir() {
        return None;
    }

    let mut files = fs::read_dir(path)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .collect::<Vec<_>>();
    files.sort_by_key(|entry| entry.file_name().to_string_lossy().to_string());

    if files.is_empty() {
        return None;
    }

    let mut hasher = Sha256::new();
    hasher.update(b"market-conditions-snapshot-v1");
    for entry in &files {
        let name = entry.file_name().to_string_lossy().to_string();
        let data = fs::read(entry.path()).ok()?;
        let mut file_hasher = Sha256::new();
        file_hasher.update(&data);
        let file_digest = file_hasher.finalize();
        hasher.update((name.len() as u32).to_le_bytes());
        hasher.update(name.as_bytes());
        hasher.update(file_digest);
    }

    let snapshot_digest = hasher.finalize().to_vec();
    let snapshot_hash_hex = hex::encode(&snapshot_digest);

    let mut payload_hasher = Sha256::new();
    payload_hasher.update(b"market-snapshot-signature-v1");
    payload_hasher.update(command_id.as_bytes());
    payload_hasher.update(&snapshot_digest);
    let payload = payload_hasher.finalize().to_vec();
    let snapshot_signature_hex = Some(hex::encode(sign_payload_for_actor(actor, &payload)));

    Some(MarketSnapshot {
        source_path: folder.to_string(),
        file_count: files.len(),
        snapshot_hash_hex,
        snapshot_signature_hex,
    })
}

fn render_ascii_barcode(payload: &str) -> String {
    let mut out = String::new();
    out.push('|');
    out.push('|');
    for bit in payload_to_bits(payload) {
        out.push(if bit { '|' } else { ' ' });
    }
    out.push('|');
    out.push('|');
    out
}

fn render_barcode_svg(payload: &str) -> String {
    let bits = payload_to_bits(payload);
    let bar_w = 2usize;
    let height = 96usize;
    let margin = 12usize;
    let width = margin * 2 + bits.len() * bar_w;
    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
        width, height, width, height
    ));
    svg.push_str(&format!(
        "<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"white\"/>",
        width, height
    ));

    for (i, bit) in bits.iter().enumerate() {
        if *bit {
            let x = margin + i * bar_w;
            svg.push_str(&format!(
                "<rect x=\"{}\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"black\"/>",
                x, bar_w, height
            ));
        }
    }

    svg.push_str("</svg>");
    svg
}

fn payload_to_bits(payload: &str) -> Vec<bool> {
    let mut bits = Vec::new();
    bits.extend([true, false, true, false, true, false, true]);
    for byte in payload.bytes() {
        for shift in (0..8).rev() {
            bits.push(((byte >> shift) & 1) == 1);
        }
        bits.push(false);
    }
    bits.extend([true, true, false, true, true]);
    bits
}

