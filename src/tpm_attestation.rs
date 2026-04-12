#![deny(unsafe_code)]

use crate::failure_axis::{FailureAxis, SystemHalt};
use crate::trusted_time::TrustedTime;
use sha2::{Digest, Sha256};

const SIM_QUOTE_MAGIC: &[u8] = b"SIMTPM1";
const SIM_AK_LABEL: &[u8] = b"simulated-ak";

pub trait TpmAttestor: Send {
    fn pcr_index(&self) -> u32;
    fn extend_pcr(&mut self, record_hash: &[u8]) -> Result<(), SystemHalt>;
    fn quote(&mut self, nonce: &[u8]) -> Result<(Vec<u8>, Vec<u8>), SystemHalt>;
}

#[derive(Debug, Clone)]
pub struct SimulatedTpmAttestor {
    pcr_index: u32,
    pcr_state: Vec<u8>,
}

impl SimulatedTpmAttestor {
    pub fn new(pcr_index: u32) -> Self {
        Self {
            pcr_index,
            pcr_state: vec![0u8; 32],
        }
    }
}

impl TpmAttestor for SimulatedTpmAttestor {
    fn pcr_index(&self) -> u32 {
        self.pcr_index
    }

    fn extend_pcr(&mut self, record_hash: &[u8]) -> Result<(), SystemHalt> {
        let mut hasher = Sha256::new();
        hasher.update(&self.pcr_state);
        hasher.update(record_hash);
        self.pcr_state = hasher.finalize().to_vec();
        Ok(())
    }

    fn quote(&mut self, nonce: &[u8]) -> Result<(Vec<u8>, Vec<u8>), SystemHalt> {
        let attest = build_sim_quote(self.pcr_index, nonce, &self.pcr_state);
        let mut sig_hasher = Sha256::new();
        sig_hasher.update(&attest);
        sig_hasher.update(SIM_AK_LABEL);
        let signature = sig_hasher.finalize().to_vec();
        Ok((attest, signature))
    }
}

pub fn build_attestation_nonce(
    decision_hash: &[u8],
    chain_hash: &[u8],
    time: &TrustedTime,
) -> Vec<u8> {
    build_attestation_nonce_with_context(decision_hash, chain_hash, time, &[])
}

pub fn build_attestation_nonce_with_context(
    decision_hash: &[u8],
    chain_hash: &[u8],
    time: &TrustedTime,
    context_hash: &[u8],
) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(decision_hash);
    hasher.update(chain_hash);
    hasher.update(time.wall_time_ns.to_le_bytes());
    hasher.update(time.monotonic_ns.to_le_bytes());
    hasher.update((context_hash.len() as u32).to_le_bytes());
    hasher.update(context_hash);
    hasher.finalize().to_vec()
}

pub fn verify_quote_nonce_and_signature(
    attest: &[u8],
    signature: &[u8],
    expected_nonce: &[u8],
) -> Result<(), String> {
    let parsed = parse_sim_quote(attest)?;
    if parsed.nonce != expected_nonce {
        return Err("ERR_NONCE_MISMATCH".to_string());
    }

    let mut sig_hasher = Sha256::new();
    sig_hasher.update(attest);
    sig_hasher.update(SIM_AK_LABEL);
    let expected_sig = sig_hasher.finalize().to_vec();
    if expected_sig != signature {
        return Err("ERR_TPM_SIGNATURE_INVALID".to_string());
    }

    Ok(())
}

pub fn tpm_unavailable(msg: &str) -> SystemHalt {
    SystemHalt::new(FailureAxis::TpmUnavailable, msg)
}

#[derive(Debug, Clone)]
struct ParsedQuote {
    nonce: Vec<u8>,
}

fn build_sim_quote(pcr_index: u32, nonce: &[u8], pcr_state_hash: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(SIM_QUOTE_MAGIC);
    out.extend(pcr_index.to_le_bytes());
    out.extend((nonce.len() as u32).to_le_bytes());
    out.extend(nonce);
    out.extend((pcr_state_hash.len() as u32).to_le_bytes());
    out.extend(pcr_state_hash);
    out
}

fn parse_sim_quote(attest: &[u8]) -> Result<ParsedQuote, String> {
    let mut idx = 0usize;
    if attest.len() < SIM_QUOTE_MAGIC.len() + 12 {
        return Err("ERR_TPM_ATTEST_MALFORMED".to_string());
    }
    if &attest[..SIM_QUOTE_MAGIC.len()] != SIM_QUOTE_MAGIC {
        return Err("ERR_TPM_ATTEST_MAGIC".to_string());
    }
    idx += SIM_QUOTE_MAGIC.len();

    idx += 4; // pcr index
    let nonce_len =
        u32::from_le_bytes(attest[idx..idx + 4].try_into().map_err(|_| "ERR_TPM_ATTEST_LEN")?)
            as usize;
    idx += 4;
    if attest.len() < idx + nonce_len + 4 {
        return Err("ERR_TPM_ATTEST_NONCE_LEN".to_string());
    }
    let nonce = attest[idx..idx + nonce_len].to_vec();

    Ok(ParsedQuote { nonce })
}
