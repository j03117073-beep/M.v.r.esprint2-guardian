#![deny(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriEntity {
    EntityA,
    EntityB,
    EntityC,
}

#[derive(Debug, Clone, Copy)]
pub struct TriEntityFrame {
    pub active_entity: TriEntity,
    pub influences: [u8; 3],
}

/// Deterministic A->B->C scheduler to ensure each domain is serviced.
#[derive(Debug, Clone, Copy)]
pub struct TriEntityScheduler {
    phase: u8,
}

impl TriEntityScheduler {
    pub fn new() -> Self {
        Self { phase: 0 }
    }

    pub fn next_frame(&mut self, influences: [u8; 3]) -> TriEntityFrame {
        let active_entity = match self.phase {
            0 => TriEntity::EntityA,
            1 => TriEntity::EntityB,
            _ => TriEntity::EntityC,
        };
        self.phase = (self.phase + 1) % 3;
        TriEntityFrame {
            active_entity,
            influences,
        }
    }
}
