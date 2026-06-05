# ternary-resonance

**Resonance and natural frequency analysis — sympathetic vibration in ternary state spaces.**

Everything resonates. A guitar string, a building in an earthquake, a crowd at a concert — when you excite something near its natural frequency, it responds disproportionately. `ternary-resonance` models this phenomenon for ternary systems: agents with resonant frequencies, coupling strengths, and harmonic series that interact, propagate, and eventually settle. The outputs quantize to {-1, 0, +1}, but the *internals* use continuous values — because real resonance doesn't live in three discrete steps.

This crate was born from a discovery: discrete ternary values alone couldn't capture true resonance behavior. The rigging experiment showed that fractional amplitudes and continuous coupling were necessary to model sympathetic vibration realistically. So `ternary-resonance` works in floating-point internally and quantizes at the boundary.

## Why This Matters

Most ternary crates in the ecosystem operate on pure {-1, 0, +1} signals. That's elegant but physically incomplete. Resonance is inherently continuous — a string doesn't vibrate at exactly three amplitudes. This crate bridges the gap: it uses real physics (Lorentzian response profiles, Q factors, damping ratios, harmonic series) internally, then quantizes to ternary when you need a discrete signal. You get physical realism *and* ternary compatibility.

The practical payoff is huge for anything that needs to feel "alive." A group of coupled ternary agents that resonate with each other produces emergent behavior — synchronized oscillations, traveling waves, cascade failures — that pure discrete logic can't achieve. This is the crate that makes ternary systems breathe.

## What's Inside

### Resonance

- **`Resonance::new(frequency, coupling) → Self`** — Create a resonant agent with frequency and coupling strength.
- **`excite(force)`** — Apply an external force, scaled by coupling.
- **`tick(dt)`** — Advance phase and apply damping.
- **`output() → f64`** — Current amplitude × sin(phase).
- **`output_ternary() → i8`** — Quantized to {-1, 0, +1} (threshold ±0.5).
- **`is_active() → bool`** — Non-negligible amplitude?
- **`energy() → f64`** — Amplitude squared.

### ResonantFrequency

- **`ResonantFrequency::new(base, q_factor) → Self`** — Natural frequency with sharpness control.
- **`bandwidth() → f64`** — How wide the resonance peak is (base / Q).
- **`response_at(freq) → f64`** — Lorentzian response profile (1.0 at peak, falling off).
- **`resonates_with(freq, threshold) → bool`** — Is a frequency "close enough" to resonate?

### CouplingStrength

- **`CouplingStrength::new(value)`** / **`none()`** / **`full()`** — Create coupling values (clamped 0–1).
- **`transfer(force) → f64`** — How much force actually passes through.
- **`combine(other) → CouplingStrength`** — Geometric mean of two couplings.
- **`is_coupled() → bool`** — Any coupling at all?

### ResonanceCascade

- **`ResonanceCascade::new(agent_count, damping, threshold) → Self`** — Create a network of agents with a coupling matrix.
- **`set_agent(idx, frequency, amplitude)`** — Configure an agent.
- **`set_coupling(from, to, strength)`** — Wire up the network.
- **`excite(idx, force)`** — Kick an agent.
- **`step()`** — Propagate energy one tick through the coupling matrix.
- **`run(steps)`** — Run N steps.
- **`total_energy() → f64`** — Sum of all amplitude squares.
- **`active_count() → usize`** — Agents above threshold.
- **`is_settled() → bool`** — Has the cascade died out?

### DampingFactor

- **`DampingFactor::new(ratio)`** — Create with specific damping (0 = none, 1 = instant death).
- **`DampingFactor::critical()`** / **`underdamped()`** / **`overdamped()`** — Physics-inspired presets.
- **`apply(value) → f64`** — Apply one step of damping.
- **`decay_steps(initial, threshold) → u32`** — How many steps until a value falls below threshold.

### HarmonicSeries

- **`HarmonicSeries::new(fundamental, harmonics) → Self`** — Create with 1/n amplitude falloff.
- **`frequency(n) → f64`** / **`amplitude(n) → f64`** — Get specific harmonic.
- **`total_energy() → f64`** — Energy across all harmonics.
- **`set_amplitude(n, amp)`** — Customize a harmonic's strength.
- **`output_at_phase(phase) → f64`** — Composite waveform at a given phase.
- **`output_ternary(phase) → i8`** — Quantized composite output.
- **`nearest_harmonic(freq) → usize`** — Which harmonic is closest to a given frequency.

## Quick Example

```rust
use ternary_resonance::{
    Resonance, ResonantFrequency, CouplingStrength,
    ResonanceCascade, DampingFactor, HarmonicSeries,
};

// Single resonant agent
let mut r = Resonance::new(2.0, 0.8); // 2 Hz, strong coupling
r.excite(5.0);
for _ in 0..100 {
    r.tick(0.01);
    print!("{} ", r.output_ternary());
}
println!();

// Resonant frequency analysis
let rf = ResonantFrequency::new(10.0, 5.0);
println!("Bandwidth: {:.1} Hz", rf.bandwidth());
println!("Response at 10 Hz: {:.3}", rf.response_at(10.0)); // 1.0
println!("Response at 15 Hz: {:.3}", rf.response_at(15.0)); // much less

// Cascade: 5 agents, chain-coupled
let mut cascade = ResonanceCascade::new(5, 0.1, 0.01);
for i in 0..4 {
    cascade.set_coupling(i, i + 1, 0.7);
}
cascade.excite(0, 10.0);
cascade.run(20);
println!("Active agents: {}/5", cascade.active_count());
println!("Total energy: {:.3}", cascade.total_energy());

// Harmonic series
let hs = HarmonicSeries::new(1.0, 8);
println!("Total harmonic energy: {:.3}", hs.total_energy());
println!("Nearest harmonic to 3.1 Hz: {}", hs.nearest_harmonic(3.1)); // 3
```

## The Deeper Truth

The `Resonance` struct is doing classical driven oscillation: phase advances proportionally to frequency, amplitude decays by damping, and output is `A·sin(φ)`. Nothing exotic. But the magic is in the `coupling` field — it determines how much of an external force actually gets absorbed. A coupling of 1.0 means the agent accepts the full force. A coupling of 0.1 means it barely notices. This is the parameter that turns a collection of independent oscillators into a *connected system*.

The `ResonantFrequency` type uses a Lorentzian response profile — the same mathematical shape that describes atomic spectral lines and RLC circuit impedance peaks. The `q_factor` controls sharpness: a high-Q resonator responds strongly only at its exact frequency (like a tuning fork), while a low-Q resonator responds to a broad range (like a snare drum). The `resonates_with` method lets you ask "will this thing ring if I shake it at this frequency?" — a question that's fundamental to understanding how energy flows through coupled systems.

The `ResonanceCascade` is the crown jewel. It's a full coupling-matrix simulation: each agent's amplitude is updated based on the weighted sum of all other agents' amplitudes, filtered by the coupling matrix and the threshold. This is where emergence lives. Chain-coupled agents (0→1→2→3→4) produce traveling waves. Fully-connected agents (everyone coupled to everyone) produce synchronized oscillation or chaotic interference, depending on parameters. A ring of agents can produce standing waves. These aren't programmed behaviors — they *emerge* from the physics.

The `DampingFactor` deserves attention for its `decay_steps` method alone. It tells you exactly how many ticks it takes for a value to fall below a threshold, given exponential decay. This is incredibly useful for planning: "if I excite the system now, how long until it's quiet enough to excite again?" That's the kind of question you need answered when building responsive interactive systems.

The `HarmonicSeries` is the bridge between pure tones and rich timbres. The default 1/n amplitude falloff produces a sawtooth-like spectrum — bright and buzzy. But `set_amplitude` lets you sculpt the harmonic content. Remove the odd harmonics and you get a square wave. Remove everything above the 3rd and you get something warm and flute-like. The `output_at_phase` method renders the composite waveform, and `output_ternary` quantizes it. This is how you go from a simple fundamental frequency to a complex ternary signal with timbral character.

## Use Cases

1. **Interactive installations** — Couple physical sensors to ternary resonators. When someone approaches, they excite the system; the cascade propagates through the network, driving audio and visuals.

2. **Procedural audio synthesis** — Use `HarmonicSeries` to build rich timbres from ternary fundamentals. Different amplitude profiles create different "instruments."

3. **Network simulation** — Model information propagation through a social network using `ResonanceCascade`. Each agent is a person, coupling is influence, and the cascade shows how a signal spreads.

4. **Rhythmic pattern generation** — Couple resonators at harmonic ratios (1:2, 2:3, 3:4) and quantize their outputs to ternary. The interference patterns produce complex, evolving rhythms.

5. **Physics education** — Demonstrate resonance, coupling, damping, and cascade behavior in a clean, inspectable system. Students can see the coupling matrix, watch energy propagate, and tweak parameters in real-time.

## See Also

- **[ternary-harmonic](https://github.com/clarkeressel/ternary-harmonic)** — Harmonic analysis in ternary space
- **[ternary-wave](https://github.com/clarkeressel/ternary-wave)** — Ternary signal waveforms and oscillators
- **[ternary-echo](https://github.com/clarkeressel/ternary-echo)** — Delay and echo effects for ternary signals
- **[ternary-rack](https://github.com/clarkeressel/ternary-rack)** — Modular synth-style composition with ternary signals
- **[ternary-phase](https://github.com/clarkeressel/ternary-phase)** — Phase relationships and interference in ternary signals

## Install

```toml
[dependencies]
ternary-resonance = "0.1"
```

## License

MIT
