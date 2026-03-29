#![deny(unsafe_code)]

use crate::sprint1::CanonicalBatch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MvreIngestAck {
    pub accepted: bool,
    pub records_total: usize,
    pub final_chain_hash: String,
}

#[derive(Debug, Default)]
pub struct MvreControlCenter {
    last_seen_chain_hash: Option<String>,
}

impl MvreControlCenter {
    pub fn consume_canonical_batch(&mut self, batch: &CanonicalBatch) -> MvreIngestAck {
        self.last_seen_chain_hash = Some(batch.final_chain_hash.clone());
        MvreIngestAck {
            accepted: true,
            records_total: batch.records_total,
            final_chain_hash: batch.final_chain_hash.clone(),
        }
    }

    pub fn last_seen_chain_hash(&self) -> Option<&str> {
        self.last_seen_chain_hash.as_deref()
    }
}

