//! # ternary-resonance
//!
//! Resonance and sympathetic vibration between agents in ternary state spaces.
//! Uses fractional values for realistic resonance modeling — inspired by the
//! rigging experiment findings where discrete {-1, 0, +1} alone couldn't
//! capture true resonance phenomena.

#![forbid(unsafe_code)]

/// Sympathetic response: when one agent vibrates, nearby agents respond.
#[derive(Debug, Clone)]
pub struct Resonance {
    /// Coupling strength (0.0..1.0).
    pub coupling: f64,
    /// Resonant frequency of this agent.
    pub frequency: f64,
    /// Current amplitude.
    pub amplitude: f64,
    /// Damping applied each tick (0.0..1.0).
    pub damping: f64,
    /// Phase offset in radians.
    pub phase: f64,
}

impl Resonance {
    pub fn new(frequency: f64, coupling: f64) -> Self {
        Self { coupling: coupling.clamp(0.0, 1.0), frequency, amplitude: 0.0, damping: 0.01, phase: 0.0 }
    }

    /// Excite this resonance with an external force.
    pub fn excite(&mut self, force: f64) {
        self.amplitude += force * self.coupling;
    }

    /// Tick: apply damping and phase advance.
    pub fn tick(&mut self, dt: f64) {
        self.phase += self.frequency * dt * 2.0 * std::f64::consts::PI;
        self.amplitude *= 1.0 - self.damping;
        if self.amplitude.abs() < 1e-10 {
            self.amplitude = 0.0;
        }
    }

    /// Current output value (amplitude * sin(phase)).
    pub fn output(&self) -> f64 {
        self.amplitude * (self.phase).sin()
    }

    /// Quantize output to ternary {-1, 0, +1}.
    pub fn output_ternary(&self) -> i8 {
        let v = self.output();
        if v > 0.5 { 1 } else if v < -0.5 { -1 } else { 0 }
    }

    /// Is this resonance active (non-negligible amplitude)?
    pub fn is_active(&self) -> bool {
        self.amplitude.abs() > 1e-6
    }

    /// Energy: proportional to amplitude squared.
    pub fn energy(&self) -> f64 {
        self.amplitude * self.amplitude
    }
}

/// Natural frequency of an agent — what makes it "ring."
#[derive(Debug, Clone, Copy)]
pub struct ResonantFrequency {
    /// Base frequency in Hz (or abstract units).
    pub base: f64,
    /// Q factor: sharpness of resonance (higher = narrower bandwidth).
    pub q_factor: f64,
}

impl ResonantFrequency {
    pub fn new(base: f64, q_factor: f64) -> Self {
        Self { base, q_factor: q_factor.max(0.1) }
    }

    /// Bandwidth: how wide the resonance peak is.
    pub fn bandwidth(&self) -> f64 {
        self.base / self.q_factor
    }

    /// Response at a given frequency — Lorentzian profile.
    pub fn response_at(&self, freq: f64) -> f64 {
        let half_bw = self.bandwidth() / 2.0;
        let delta = freq - self.base;
        1.0 / (1.0 + (delta / half_bw).powi(2))
    }

    /// Is a frequency within the resonance bandwidth?
    pub fn resonates_with(&self, freq: f64, threshold: f64) -> bool {
        self.response_at(freq) >= threshold
    }
}

/// How strongly agents influence each other.
#[derive(Debug, Clone, Copy)]
pub struct CouplingStrength {
    /// Coupling value (0.0 = independent, 1.0 = fully coupled).
    pub value: f64,
}

impl CouplingStrength {
    pub fn new(value: f64) -> Self {
        Self { value: value.clamp(0.0, 1.0) }
    }

    /// No coupling.
    pub fn none() -> Self {
        Self { value: 0.0 }
    }

    /// Full coupling.
    pub fn full() -> Self {
        Self { value: 1.0 }
    }

    /// Effective force transferred.
    pub fn transfer(&self, force: f64) -> f64 {
        force * self.value
    }

    /// Combine two coupling strengths (geometric mean).
    pub fn combine(&self, other: &CouplingStrength) -> CouplingStrength {
        CouplingStrength::new((self.value * other.value).sqrt())
    }

    /// Is there any coupling?
    pub fn is_coupled(&self) -> bool {
        self.value > 1e-6
    }
}

/// Chain reaction of activations propagating through agents.
#[derive(Debug, Clone)]
pub struct ResonanceCascade {
    /// Agent states: (frequency, amplitude).
    pub agents: Vec<(f64, f64)>,
    /// Coupling matrix: agents[i] influences agents[j] by this factor.
    pub coupling_matrix: Vec<Vec<f64>>,
    /// Damping factor per tick.
    pub global_damping: f64,
    /// Activation threshold — amplitude must exceed this to propagate.
    pub threshold: f64,
    /// Steps taken.
    pub steps: u64,
}

impl ResonanceCascade {
    pub fn new(agent_count: usize, global_damping: f64, threshold: f64) -> Self {
        let agents = vec![(0.0, 0.0); agent_count];
        let coupling_matrix = vec![vec![0.0; agent_count]; agent_count];
        Self { agents, coupling_matrix, global_damping, threshold, steps: 0 }
    }

    /// Set an agent's frequency and initial amplitude.
    pub fn set_agent(&mut self, idx: usize, frequency: f64, amplitude: f64) {
        if idx < self.agents.len() {
            self.agents[idx] = (frequency, amplitude);
        }
    }

    /// Set coupling from agent i to agent j.
    pub fn set_coupling(&mut self, from: usize, to: usize, strength: f64) {
        if from < self.agents.len() && to < self.agents.len() {
            self.coupling_matrix[from][to] = strength.clamp(0.0, 1.0);
        }
    }

    /// Excite a single agent.
    pub fn excite(&mut self, idx: usize, force: f64) {
        if idx < self.agents.len() {
            self.agents[idx].1 += force;
        }
    }

    /// Advance one step: propagate energy through coupling matrix.
    pub fn step(&mut self) {
        let n = self.agents.len();
        let mut new_amplitudes = vec![0.0f64; n];

        for j in 0..n {
            let mut received = 0.0;
            for i in 0..n {
                let (_freq_i, amp_i) = self.agents[i];
                if amp_i.abs() > self.threshold {
                    received += self.coupling_matrix[i][j] * amp_i;
                }
            }
            new_amplitudes[j] = (self.agents[j].1 + received) * (1.0 - self.global_damping);
        }

        for j in 0..n {
            self.agents[j].1 = new_amplitudes[j];
        }
        self.steps += 1;
    }

    /// Run N steps.
    pub fn run(&mut self, steps: u32) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// Total energy in the system.
    pub fn total_energy(&self) -> f64 {
        self.agents.iter().map(|(_, a)| a * a).sum()
    }

    /// Number of active agents (above threshold).
    pub fn active_count(&self) -> usize {
        self.agents.iter().filter(|(_, a)| a.abs() > self.threshold).count()
    }

    /// Has the cascade died out?
    pub fn is_settled(&self) -> bool {
        self.active_count() == 0
    }
}

/// Energy loss in propagation.
#[derive(Debug, Clone, Copy)]
pub struct DampingFactor {
    /// Damping ratio (0.0 = no loss, 1.0 = full loss immediately).
    pub ratio: f64,
}

impl DampingFactor {
    pub fn new(ratio: f64) -> Self {
        Self { ratio: ratio.clamp(0.0, 1.0) }
    }

    /// Apply damping to a value.
    pub fn apply(&self, value: f64) -> f64 {
        value * (1.0 - self.ratio)
    }

    /// How many steps until a value decays below threshold.
    pub fn decay_steps(&self, initial: f64, threshold: f64) -> u32 {
        if initial.abs() <= threshold || self.ratio <= 0.0 { return 0; }
        let ratio = (threshold / initial.abs()).ln();
        let per_step = (1.0 - self.ratio).ln();
        if per_step >= 0.0 { return u32::MAX; }
        (ratio / per_step).ceil().max(0.0) as u32
    }

    /// Critically damped: ratio tuned for fastest settling without overshoot.
    pub fn critical() -> Self {
        Self { ratio: 0.5 }
    }

    /// Underdamped: slow decay, oscillatory.
    pub fn underdamped() -> Self {
        Self { ratio: 0.1 }
    }

    /// Overdamped: fast decay, no oscillation.
    pub fn overdamped() -> Self {
        Self { ratio: 0.9 }
    }
}

/// Natural overtone structure of a ternary agent.
#[derive(Debug, Clone)]
pub struct HarmonicSeries {
    /// Fundamental frequency.
    pub fundamental: f64,
    /// Number of harmonics to track.
    pub harmonics: usize,
    /// Relative amplitude of each harmonic (1.0 = fundamental strength).
    pub amplitudes: Vec<f64>,
}

impl HarmonicSeries {
    pub fn new(fundamental: f64, harmonics: usize) -> Self {
        let amplitudes: Vec<f64> = (1..=harmonics).map(|n| 1.0 / n as f64).collect();
        Self { fundamental, harmonics, amplitudes }
    }

    /// Frequency of the nth harmonic (1-indexed: 1 = fundamental).
    pub fn frequency(&self, n: usize) -> f64 {
        if n == 0 || n > self.harmonics { return 0.0; }
        self.fundamental * n as f64
    }

    /// Amplitude of the nth harmonic.
    pub fn amplitude(&self, n: usize) -> f64 {
        if n == 0 || n > self.harmonics { return 0.0; }
        *self.amplitudes.get(n - 1).unwrap_or(&0.0)
    }

    /// Total energy across all harmonics.
    pub fn total_energy(&self) -> f64 {
        self.amplitudes.iter().map(|a| a * a).sum()
    }

    /// Set custom amplitude for a harmonic.
    pub fn set_amplitude(&mut self, n: usize, amp: f64) {
        if n > 0 && n <= self.harmonics {
            self.amplitudes[n - 1] = amp;
        }
    }

    /// Composite output at a given phase.
    pub fn output_at_phase(&self, phase: f64) -> f64 {
        let mut val = 0.0;
        for n in 1..=self.harmonics {
            val += self.amplitude(n) * (phase * n as f64).sin();
        }
        val
    }

    /// Quantize composite output to ternary.
    pub fn output_ternary(&self, phase: f64) -> i8 {
        let v = self.output_at_phase(phase);
        if v > 0.5 { 1 } else if v < -0.5 { -1 } else { 0 }
    }

    /// Find which harmonic is closest to a given frequency.
    pub fn nearest_harmonic(&self, freq: f64) -> usize {
        let mut best = 1;
        let mut best_dist = (self.frequency(1) - freq).abs();
        for n in 2..=self.harmonics {
            let dist = (self.frequency(n) - freq).abs();
            if dist < best_dist {
                best_dist = dist;
                best = n;
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resonance_excite_and_tick() {
        let mut r = Resonance::new(1.0, 0.5);
        r.excite(2.0);
        assert!(r.amplitude > 0.0);
        r.tick(0.1);
        assert!(r.amplitude < 1.0); // damped
    }

    #[test]
    fn resonance_output_ternary() {
        let mut r = Resonance::new(1.0, 1.0);
        r.excite(10.0); // large amplitude
        let t = r.output_ternary();
        assert!(t == -1 || t == 0 || t == 1);
    }

    #[test]
    fn resonance_is_active() {
        let mut r = Resonance::new(1.0, 1.0);
        assert!(!r.is_active());
        r.excite(1.0);
        assert!(r.is_active());
    }

    #[test]
    fn resonance_energy() {
        let mut r = Resonance::new(1.0, 1.0);
        r.amplitude = 3.0;
        assert!((r.energy() - 9.0).abs() < 0.001);
    }

    #[test]
    fn resonant_frequency_response() {
        let rf = ResonantFrequency::new(10.0, 10.0);
        let peak = rf.response_at(10.0);
        assert!((peak - 1.0).abs() < 0.001);
        let off = rf.response_at(15.0);
        assert!(off < 0.5);
    }

    #[test]
    fn resonant_frequency_bandwidth() {
        let rf = ResonantFrequency::new(100.0, 10.0);
        assert!((rf.bandwidth() - 10.0).abs() < 0.001);
    }

    #[test]
    fn resonant_frequency_resonates_with() {
        let rf = ResonantFrequency::new(10.0, 5.0);
        assert!(rf.resonates_with(10.0, 0.9));
        assert!(!rf.resonates_with(20.0, 0.9));
    }

    #[test]
    fn coupling_strength_clamp() {
        let cs = CouplingStrength::new(1.5);
        assert!((cs.value - 1.0).abs() < 0.001);
        let cs2 = CouplingStrength::new(-0.5);
        assert!((cs2.value - 0.0).abs() < 0.001);
    }

    #[test]
    fn coupling_strength_transfer() {
        let cs = CouplingStrength::new(0.5);
        assert!((cs.transfer(4.0) - 2.0).abs() < 0.001);
    }

    #[test]
    fn coupling_strength_combine() {
        let a = CouplingStrength::new(0.25);
        let b = CouplingStrength::new(0.25);
        let combined = a.combine(&b);
        assert!((combined.value - 0.25).abs() < 0.001);
    }

    #[test]
    fn cascade_propagation() {
        let mut cascade = ResonanceCascade::new(3, 0.1, 0.01);
        cascade.set_coupling(0, 1, 0.8);
        cascade.set_coupling(1, 2, 0.8);
        cascade.excite(0, 10.0);
        cascade.step();
        assert!(cascade.agents[1].1.abs() > 0.0); // propagated
        cascade.step();
        assert!(cascade.agents[2].1.abs() > 0.0); // further propagation
    }

    #[test]
    fn cascade_settles() {
        let mut cascade = ResonanceCascade::new(2, 0.5, 0.01);
        cascade.set_coupling(0, 1, 0.5);
        cascade.excite(0, 1.0);
        cascade.run(50);
        assert!(cascade.is_settled());
    }

    #[test]
    fn cascade_total_energy() {
        let mut cascade = ResonanceCascade::new(2, 0.0, 0.0);
        cascade.set_agent(0, 1.0, 3.0);
        cascade.set_agent(1, 2.0, 4.0);
        assert!((cascade.total_energy() - 25.0).abs() < 0.001);
    }

    #[test]
    fn damping_factor_apply() {
        let d = DampingFactor::new(0.1);
        assert!((d.apply(10.0) - 9.0).abs() < 0.001);
    }

    #[test]
    fn damping_factor_decay_steps() {
        let d = DampingFactor::new(0.5);
        let steps = d.decay_steps(1.0, 0.01);
        assert!(steps > 0);
        assert!(steps < 100);
    }

    #[test]
    fn damping_factor_presets() {
        assert!(DampingFactor::critical().ratio > 0.0);
        assert!(DampingFactor::underdamped().ratio < DampingFactor::critical().ratio);
        assert!(DampingFactor::overdamped().ratio > DampingFactor::critical().ratio);
    }

    #[test]
    fn harmonic_series_frequencies() {
        let hs = HarmonicSeries::new(10.0, 5);
        assert!((hs.frequency(1) - 10.0).abs() < 0.001);
        assert!((hs.frequency(3) - 30.0).abs() < 0.001);
        assert!((hs.frequency(0)).abs() < 0.001); // invalid
    }

    #[test]
    fn harmonic_series_amplitudes() {
        let hs = HarmonicSeries::new(1.0, 4);
        assert!((hs.amplitude(1) - 1.0).abs() < 0.001);
        assert!((hs.amplitude(2) - 0.5).abs() < 0.001);
        assert!((hs.amplitude(4) - 0.25).abs() < 0.001);
    }

    #[test]
    fn harmonic_series_total_energy() {
        let hs = HarmonicSeries::new(1.0, 2);
        let expected = 1.0_f64.powi(2) + 0.5_f64.powi(2);
        assert!((hs.total_energy() - expected).abs() < 0.001);
    }

    #[test]
    fn harmonic_series_output_ternary() {
        let hs = HarmonicSeries::new(1.0, 3);
        let t = hs.output_ternary(0.0);
        assert!(t == -1 || t == 0 || t == 1);
    }

    #[test]
    fn harmonic_series_nearest() {
        let hs = HarmonicSeries::new(10.0, 5);
        assert_eq!(hs.nearest_harmonic(10.0), 1);
        assert_eq!(hs.nearest_harmonic(20.0), 2);
        assert_eq!(hs.nearest_harmonic(39.0), 4); // 39 is closer to 40 than 30
    }

    #[test]
    fn resonance_coupling_clamp() {
        let r = Resonance::new(1.0, 2.0);
        assert!((r.coupling - 1.0).abs() < 0.001);
    }
}
