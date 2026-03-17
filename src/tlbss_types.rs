#![deny(unsafe_code)]

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryState {
    Zero = 0,
    One = 1,
}

impl BinaryState {
    pub fn from_u8(value: u8) -> Result<Self, crate::failure_axis::SystemHalt> {
        match value {
            0 => Ok(BinaryState::Zero),
            1 => Ok(BinaryState::One),
            _ => Err(crate::failure_axis::SystemHalt::new(
                crate::failure_axis::FailureAxis::InternalInvariantBreach,
                "Invalid BinaryState",
            )),
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone)]
pub struct SubstrateNode {
    pub charge: u64,
    pub masked_signal: u8,
    pub stable_ticks: u8,
    pub entity_a: Option<Box<SubstrateNode>>,
    pub entity_b: Option<Box<SubstrateNode>>,
    pub entity_c: Option<Box<SubstrateNode>>,
}

impl SubstrateNode {
    pub fn new(raw_signal: u8) -> Self {
        Self {
            charge: 0,
            masked_signal: raw_signal ^ 0x5A,
            stable_ticks: 0,
            entity_a: None,
            entity_b: None,
            entity_c: None,
        }
    }

    pub fn validate(&self) -> Result<(), crate::failure_axis::SystemHalt> {
        if (self.masked_signal ^ 0x5A) > 1 {
            return Err(crate::failure_axis::SystemHalt::new(
                crate::failure_axis::FailureAxis::InternalInvariantBreach,
                "Masked signal violates binary invariant",
            ));
        }
        Ok(())
    }

    pub fn stability_invariant_met(&self) -> bool {
        self.stable_ticks >= 3
    }
}
