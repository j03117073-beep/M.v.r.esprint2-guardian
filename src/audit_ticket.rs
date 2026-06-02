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

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const AUDIT_TICKET_SEED: &str = "M.V.R.ESPRINT1-AUDIT-TICKET-V1";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepositoryInfo {
    pub name: String,
    pub commit: String,
    pub tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BaselineInfo {
    pub commit: String,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactEntry {
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub repository: RepositoryInfo,
    pub baseline: BaselineInfo,
    pub generated_at: u64,
    pub artifacts: Vec<ArtifactEntry>,
    pub manifest_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditTicket {
    pub ticket_version: String,
    pub manifest: Manifest,
    pub ticket_signature: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationReport {
    pub outcome: String,
    pub details: Vec<String>,
    pub baseline_commit: String,
    pub baseline_tag: String,
    pub mismatches: Vec<String>,
    pub missing_files: Vec<String>,
}

pub fn generate_manifest<P: AsRef<Path>>(
    repo_root: P,
    baseline_commit: String,
    baseline_tag: String,
) -> Result<Manifest, String> {
    let repo_root = repo_root.as_ref();
    let repository = repository_info(repo_root)?;
    let mut artifacts = Vec::new();
    let exclusions = excluded_paths();

    walk_tree(repo_root, repo_root, &exclusions, &mut artifacts)
        .map_err(|e| format!("Failed to walk repository: {}", e))?;

    artifacts.sort_by(|a, b| a.path.cmp(&b.path));

    let mut manifest = Manifest {
        repository,
        baseline: BaselineInfo {
            commit: baseline_commit,
            tag: baseline_tag,
        },
        generated_at: current_unix_timestamp()?,
        artifacts,
        manifest_hash: String::new(),
    };

    manifest.manifest_hash = compute_manifest_hash(&manifest)?;
    Ok(manifest)
}

pub fn create_audit_ticket(manifest: &Manifest, summary: String) -> Result<AuditTicket, String> {
    let ticket_signature = compute_ticket_signature(&manifest.manifest_hash, &manifest.baseline.commit, &manifest.baseline.tag);
    Ok(AuditTicket {
        ticket_version: "1.0".to_string(),
        manifest: manifest.clone(),
        ticket_signature,
        summary,
    })
}

pub fn verify_ticket<P: AsRef<Path>>(ticket: &AuditTicket, repo_root: P) -> VerificationReport {
    let repo_root = repo_root.as_ref();
    let mut details = Vec::new();
    let mut mismatches = Vec::new();
    let mut missing_files = Vec::new();
    let mut outcome = "PASS".to_string();

    if let Err(e) = verify_manifest_hash(&ticket.manifest) {
        details.push(format!("Manifest hash mismatch: {}", e));
        outcome = "FAIL".to_string();
    }

    let expected_signature = compute_ticket_signature(
        &ticket.manifest.manifest_hash,
        &ticket.manifest.baseline.commit,
        &ticket.manifest.baseline.tag,
    );
    if ticket.ticket_signature != expected_signature {
        details.push("Ticket signature invalid".to_string());
        outcome = "FAIL".to_string();
    }

    match repository_info(repo_root) {
        Ok(current_repo) => {
            if current_repo.commit != ticket.manifest.baseline.commit {
                details.push(format!(
                    "Repository commit mismatch: repo={}, ticket={}",
                    current_repo.commit, ticket.manifest.baseline.commit
                ));
                outcome = "FAIL".to_string();
            }
            if let Some(current_tag) = current_repo.tag {
                if current_tag != ticket.manifest.baseline.tag {
                    details.push(format!(
                        "Repository tag mismatch: repo={}, ticket={}",
                        current_tag, ticket.manifest.baseline.tag
                    ));
                    if outcome != "FAIL" {
                        outcome = "PASS WITH CONDITIONS".to_string();
                    }
                }
            } else {
                details.push("No git tag found for current HEAD".to_string());
                if outcome != "FAIL" {
                    outcome = "PASS WITH CONDITIONS".to_string();
                }
            }
        }
        Err(err) => {
            details.push(format!("Unable to read repository identity: {}", err));
            if outcome != "FAIL" {
                outcome = "PASS WITH CONDITIONS".to_string();
            }
        }
    }

    let (missing, hashes) = verify_manifest_entries(repo_root, &ticket.manifest);
    if !missing.is_empty() {
        missing_files.extend(missing.into_iter());
        details.push("Some files from the manifest are missing in the repository".to_string());
        outcome = "FAIL".to_string();
    }
    if !hashes.is_empty() {
        mismatches.extend(hashes.into_iter());
        details.push("Some file hashes do not match the manifest".to_string());
        outcome = "FAIL".to_string();
    }

    if details.is_empty() {
        details.push("Audit ticket verification succeeded".to_string());
    }

    VerificationReport {
        outcome,
        details,
        baseline_commit: ticket.manifest.baseline.commit.clone(),
        baseline_tag: ticket.manifest.baseline.tag.clone(),
        mismatches,
        missing_files,
    }
}

pub fn current_unix_timestamp() -> Result<u64, String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?;
    Ok(now.as_secs())
}

pub fn repository_info(repo_root: &Path) -> Result<RepositoryInfo, String> {
    let name = repo_root
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("m_v_r_esprint1")
        .to_string();

    let commit = git_command(repo_root, &["rev-parse", "--short", "HEAD"])?;
    let tag = git_command(repo_root, &["tag", "--points-at", "HEAD"]).ok();
    let tag = tag.and_then(|text| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.lines().next().unwrap_or("").to_string())
        }
    });

    Ok(RepositoryInfo { name, commit, tag })
}

pub fn compute_manifest_hash(manifest: &Manifest) -> Result<String, String> {
    let mut copy = manifest.clone();
    copy.manifest_hash = String::new();
    let payload = serde_json::to_string(&copy).map_err(|e| e.to_string())?;
    Ok(hex::encode(Sha256::digest(payload.as_bytes())))
}

pub fn compute_ticket_signature(manifest_hash: &str, baseline_commit: &str, baseline_tag: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(AUDIT_TICKET_SEED.as_bytes());
    hasher.update(manifest_hash.as_bytes());
    hasher.update(b"||");
    hasher.update(baseline_commit.as_bytes());
    hasher.update(b"||");
    hasher.update(baseline_tag.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn verify_manifest_hash(manifest: &Manifest) -> Result<(), String> {
    let actual = compute_manifest_hash(manifest)?;
    if actual != manifest.manifest_hash {
        Err(format!("expected={}, actual={}", manifest.manifest_hash, actual))
    } else {
        Ok(())
    }
}

pub fn verify_manifest_entries(repo_root: &Path, manifest: &Manifest) -> (Vec<String>, Vec<String>) {
    let mut missing = Vec::new();
    let mut mismatches = Vec::new();

    for entry in &manifest.artifacts {
        let candidate = repo_root.join(&entry.path);
        if !candidate.exists() {
            missing.push(entry.path.clone());
            continue;
        }
        match compute_file_sha256(&candidate) {
            Ok(found_hash) => {
                if found_hash != entry.sha256 {
                    mismatches.push(format!("{}: expected {}, actual {}", entry.path, entry.sha256, found_hash));
                }
            }
            Err(err) => {
                mismatches.push(format!("{}: hash error {}", entry.path, err));
            }
        }
    }

    (missing, mismatches)
}

fn compute_file_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn walk_tree(
    repo_root: &Path,
    current: &Path,
    exclusions: &HashSet<String>,
    artifacts: &mut Vec<ArtifactEntry>,
) -> io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(repo_root).unwrap();
        if relative.components().any(|component| {
            exclusions.contains(component.as_os_str().to_string_lossy().as_ref())
        }) {
            continue;
        }

        let metadata = entry.metadata()?;
        if metadata.file_type().is_symlink() {
            continue;
        }

        if metadata.is_dir() {
            walk_tree(repo_root, &path, exclusions, artifacts)?;
        } else if metadata.is_file() {
            let sha256 = compute_file_sha256(&path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            artifacts.push(ArtifactEntry {
                path: relative.to_string_lossy().replace("\\", "/"),
                sha256,
                size: metadata.len(),
            });
        }
    }
    Ok(())
}

fn excluded_paths() -> HashSet<String> {
    [".git", "target", "node_modules", "**/.git"].iter().map(|s| s.to_string()).collect()
}

fn git_command(repo_root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|e| e.to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}
