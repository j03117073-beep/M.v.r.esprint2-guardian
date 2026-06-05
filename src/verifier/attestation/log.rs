use crate::verifier::attestation::AttestationRecord;
use std::fs;

pub struct AttestationLog {
    pub records: Vec<AttestationRecord>,
}

impl AttestationLog {
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        let data = fs::read_to_string(path.as_ref()).map_err(|e| format!("read failed: {e}"))?;
        let records: Vec<AttestationRecord> = serde_json::from_str(&data).map_err(|e| format!("json parse: {e}"))?;
        Ok(Self { records })
    }
}
