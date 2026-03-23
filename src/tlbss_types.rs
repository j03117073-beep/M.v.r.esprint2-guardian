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
//
// This software contains proprietary trade secrets and confidential
// intellectual property. Unauthorized use is strictly prohibited.


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

#[derive(Debug, Clone)]
pub struct ResonanceCurve {
    pub frequency_hz: f32,
    pub amplitude: f32,
    pub damping_factor: f32,
}

#[derive(Debug, Clone)]
pub struct RampEnvelope {
    pub max_ramp_rate_mw_per_min: f32,
    pub min_ramp_rate_mw_per_min: f32,
}

#[derive(Debug, Clone)]
pub struct SaturationCurve {
    pub max_power_mw: f32,
    pub knee_point_mw: f32,
}

#[derive(Debug, Clone)]
pub struct FRTWindow {
    pub fault_duration_ms: u32,
    pub recovery_time_ms: u32,
}

#[derive(Debug, Clone)]
pub struct PlantClassEnvelope {
    pub resonance_curve: ResonanceCurve,
    pub ramp_envelope: RampEnvelope,
    pub saturation_curve: SaturationCurve,
    pub frt_window: FRTWindow,
}

impl PlantClassEnvelope {
    pub fn for_solar() -> Self {
        Self {
            resonance_curve: ResonanceCurve {
                frequency_hz: 60.0,
                amplitude: 0.1,
                damping_factor: 0.05,
            },
            ramp_envelope: RampEnvelope {
                max_ramp_rate_mw_per_min: 10.0,
                min_ramp_rate_mw_per_min: -10.0,
            },
            saturation_curve: SaturationCurve {
                max_power_mw: 100.0,
                knee_point_mw: 80.0,
            },
            frt_window: FRTWindow {
                fault_duration_ms: 150,
                recovery_time_ms: 1000,
            },
        }
    }

    pub fn for_wind() -> Self {
        Self {
            resonance_curve: ResonanceCurve {
                frequency_hz: 60.0,
                amplitude: 0.15,
                damping_factor: 0.08,
            },
            ramp_envelope: RampEnvelope {
                max_ramp_rate_mw_per_min: 20.0,
                min_ramp_rate_mw_per_min: -20.0,
            },
            saturation_curve: SaturationCurve {
                max_power_mw: 200.0,
                knee_point_mw: 150.0,
            },
            frt_window: FRTWindow {
                fault_duration_ms: 200,
                recovery_time_ms: 1500,
            },
        }
    }

    // Add more for other plant classes
}
