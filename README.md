# ternary-resonance: Resonance and sympathetic vibration between ternary agents

Models resonance, coupling, and chain-reaction propagation between agents using fractional values for realistic resonance modeling. Inspired by the rigging experiment findings — discrete {-1, 0, +1} alone couldn't capture true resonance phenomena, so this crate uses continuous amplitudes internally and quantizes to ternary on output.

## Why This Exists

During the ternary rigging experiments, we discovered that discrete {-1, 0, +1} states couldn't model resonance properly. Real resonance is a continuous phenomenon — energy builds gradually, oscillates, and decays. An agent at state 0 that receives a small push shouldn't immediately jump to +1; it should accumulate energy and only cross the threshold when enough force is applied. This crate bridges that gap with fractional internal state and ternary output.

## Core Concepts

- **Resonance**: A single resonant agent with frequency, coupling strength, amplitude, damping, and phase. Excite it with force, tick it forward, and read its output (continuous or quantized to ternary).
- **ResonantFrequency**: Defines an agent's natural frequency and Q factor (resonance sharpness). Uses a Lorentzian response profile — peak response at the natural frequency, falling off with bandwidth.
- **CouplingStrength**: How strongly agents influence each other (0.0 = independent, 1.0 = fully coupled). Supports combining via geometric mean.
- **ResonanceCascade**: A network of agents with a coupling matrix. Energy propagates from agent to agent based on coupling strengths, with a global damping factor and activation threshold. Models chain reactions.
- **DampingFactor**: Energy loss per tick. Includes preset damping modes (underdamped, critical, overdamped) and a decay-step calculator.
- **HarmonicSeries**: The natural overtone structure of an agent — fundamental frequency plus harmonics with decreasing amplitude (1/n). Supports composite waveform output and nearest-harmonic lookup.

## Quick Start

```toml
[dependencies]
ternary-resonance = "0.1"
```

```rust
use ternary_resonance::*;

// Create a resonant agent
let mut r = Resonance::new(2.0, 0.8); // 2 Hz, 80% coupling
r.excite(5.0);                         // hit it hard

for _ in 0..10 {
    r.tick(0.1);                        // advance 0.1s
    println!("Output: {:.3} (ternary: {})", r.output(), r.output_ternary());
}

// Chain reaction
let mut cascade = ResonanceCascade::new(4, 0.1, 0.01);
cascade.set_coupling(0, 1, 0.9);
cascade.set_coupling(1, 2, 0.7);
cascade.set_coupling(2, 3, 0.5);
cascade.excite(0, 10.0);
cascade.run(20);
println!("Active agents: {}/4", cascade.active_count());
```

## API Overview

| Type | Description |
|------|-------------|
| `Resonance` | Single resonant agent with frequency, coupling, amplitude, damping, phase |
| `ResonantFrequency` | Natural frequency definition with Q factor and Lorentzian response curve |
| `CouplingStrength` | Agent-to-agent coupling factor (0.0..1.0) with combine/transfer methods |
| `ResonanceCascade` | Network of coupled agents with propagation, damping, and chain reactions |
| `DampingFactor` | Energy loss model with presets and decay-step prediction |
| `HarmonicSeries` | Overtone structure: fundamental + harmonics at 1/n amplitude |

## How It Works

Each `Resonance` agent is a simple damped oscillator: it has a frequency (how fast it oscillates), an amplitude (how strong the oscillation is), and a damping factor (how quickly the amplitude decays). When excited by an external force, the amplitude increases proportionally to the coupling strength. Each tick advances the phase and applies damping multiplicatively.

The `ResonanceCascade` models a network: a coupling matrix defines how strongly agent i influences agent j. Each step, every agent receives energy from all other agents above the activation threshold, weighted by the coupling factor. Global damping is applied after propagation. This creates chain reactions — exciting one agent can cascade through the network.

The `HarmonicSeries` models overtones: a fundamental frequency f with harmonics at 2f, 3f, etc., each at amplitude 1/n. The composite output at any phase is the sum of all harmonics' sinusoidal contributions. This is the foundation for understanding how agents with different natural frequencies can still resonate with each other through shared overtones.

The key insight from the rigging experiments: `output_ternary()` quantizes to {-1, 0, +1} only at the output boundary. Internally, everything is continuous — amplitudes are f64, phases are f64, coupling is f64. This gives real resonance behavior while maintaining the ternary interface the fleet expects.

## Known Limitations

- `Resonance` is a simplified oscillator — no true spring-mass dynamics, just amplitude accumulation and damping.
- The cascade coupling matrix is dense O(n²) — not suitable for very large agent counts without sparse optimization.
- No frequency-dependent coupling — in real physics, coupling strength varies with frequency proximity. Currently it's a constant.
- Harmonic series uses fixed 1/n amplitude falloff — real instruments have different overtone profiles.
- The activation threshold in ResonanceCascade is a simple absolute value — no direction sensitivity.
- Phase wrapping is implicit (sin handles it), but very large phase values may lose precision over millions of ticks.

## Use Cases

- **Cascade failure modeling**: Excite one agent and watch failure propagate through coupling links.
- **Sympathetic activation**: Agents with similar resonant frequencies activate together when one is stimulated.
- **Energy decay analysis**: Use DampingFactor to predict how long a perturbation takes to settle.
- **Harmonic alignment**: Use HarmonicSeries to find which agents share overtones and thus naturally synchronize.

## Ecosystem Context

Part of the SuperInstance ternary crate family. This is the physics-inspired layer — models how energy and activation propagate through the fleet. Directly informed by findings from `ternary-rigging`. Pairs with `ternary-ear` (perception of resonance) and `ternary-jam` (coordinated performance influenced by resonance).

## License

MIT
