#![deny(unsafe_code)]

//! Utilities to simulate non‑ideal sensor and communication behaviour.  These
//! functions are only used in the non‑real‑time test harness and do not
//! participate in the deterministic loop.

/// Simple linear‑congruential generator used for deterministic pseudo‑random
/// sequences.  The parameters are chosen to be the minimal standard LCG.
pub struct SimpleRng(u64);

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        // x_{n+1} = 6364136223846793005 * x_n + 1
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }

    #[inline]
    pub fn uniform_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }

    /// Gaussian noise via Box–Muller transform.
    pub fn gaussian(&mut self, mean: f64, std_dev: f64) -> f64 {
        let u1 = self.uniform_f64().max(1e-12);
        let u2 = self.uniform_f64();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        mean + z0 * std_dev
    }
}

/// Simulate measurement of frequency with additive Gaussian noise.
pub fn noisy_frequency(true_freq: f64, rng: &mut SimpleRng) -> f64 {
    rng.gaussian(true_freq, 0.01)
}

/// Simulate voltage measurement within ±5 % envelope with noise.
pub fn noisy_voltage(true_volt: f64, rng: &mut SimpleRng) -> f64 {
    rng.gaussian(true_volt, 0.02)
}

/// Random latency jitter between `min_ms` and `max_ms` inclusive.
pub fn latency_jitter(rng: &mut SimpleRng, min_ms: u64, max_ms: u64) -> u64 {
    let span = max_ms - min_ms;
    min_ms + (rng.next_u64() % (span + 1))
}
