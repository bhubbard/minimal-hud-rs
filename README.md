# minimal-hud-rs

[![Live Demo](https://img.shields.io/badge/Demo-Live%20Visualizer-brightgreen?style=for-the-badge&logo=github)](https://code.brandonhubbard.com/minimal-hud-rs/)
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue?style=for-the-badge)](LICENSE-MIT)
[![Rust 2024](https://img.shields.io/badge/Rust-2024%20Edition-orange?style=for-the-badge&logo=rust)](Cargo.toml)
[![Bevy 0.15](https://img.shields.io/badge/Bevy-0.15%20ECS-purple?style=for-the-badge&logo=bevy)](src/bevy_adapter.rs)
[![Tests](https://img.shields.io/badge/Tests-100%25%20Passing-success?style=for-the-badge&logo=checkmarx)](tests/)

A pure Rust, event-driven HUD state tracking, dynamic alert evaluation, vehicle telemetry, and dirty change-detection engine ported from the famous FiveM [ThatMadCap/minimal-hud](https://github.com/ThatMadCap/minimal-hud).

Designed for high-frequency game loops, simulators, and Bevy ECS projects requiring zero redundant DOM/canvas/GPU redraw overhead.

---

## 🎮 Interactive Live Demo

Experience the interactive web visualizer with live SVG circular meters, vehicle dashboard, audio synthesizers, and real-time render cycle profiling:

👉 **[https://code.brandonhubbard.com/minimal-hud-rs/](https://code.brandonhubbard.com/minimal-hud-rs/)** *(Mirror: [https://bhubbard.github.io/minimal-hud-rs/](https://bhubbard.github.io/minimal-hud-rs/))*

---

## 🌟 Key Architecture & Features

### 1. Vital Stat Models (`src/vitals.rs`)
- Full vital metric tracking:
  - **Health** ($0.0 \dots 100.0$)
  - **Armor** ($0.0 \dots 100.0$) with automated damage absorption and health spillover
  - **Stamina** ($0.0 \dots 100.0$)
  - **Hunger** ($0.0 \dots 100.0$)
  - **Thirst** ($0.0 \dots 100.0$)
  - **Stress** ($0.0 \dots 100.0$)
  - **Oxygen** ($0.0 \dots 100.0$, underwater breathing)
  - **VoiceProximity**:
    - `Whisper`: $1.5\text{m}$
    - `Normal`: $5.0\text{m}$
    - `Shout`: $12.0\text{m}$
    - `Custom(f32)`: arbitrary range
    - Push-to-talk state & optional radio channel
- **Dynamic Threshold Alerts**:
  - Low Health Warning ($\le 20\%$)
  - Drowning ($\le 15\%$)
  - Dehydration ($\le 10\%$)
  - Starvation ($\le 10\%$)
  - Exhaustion ($\le 5\%$)
  - High Stress ($\ge 80\%$)

### 2. Vehicle Telemetry (`src/vehicle.rs`)
- **Speed**: Scalar internal representation in SI $\text{m/s}$ with exact bidirectional conversions between $\text{mph}$, $\text{km/h}$, and $\text{m/s}$.
- **Engine RPM**: Normalized $0.0 \dots 1.0$ with redline detection ($\ge 0.85$).
- **Gears**: Reverse (`-1`), Neutral (`0`), Forward (`1..=7`).
- **Fuel Level**: $0.0 \dots 100.0\%$ with low fuel alert ($\le 15\%$).
- **Engine Health & Qualitative States**:
  - `Optimal` ($> 800.0$)
  - `MinorDamage` ($600.0 \dots 800.0$)
  - `ModerateDamage` ($300.0 \dots 600.0$)
  - `Critical` ($0.0 \dots 300.0$ — smoke & stall risk)
  - `Destroyed` ($\le 0.0$)
- **Indicator States**: Seatbelt buckled flag, headlights cycling (`Off`, `LowBeam`, `HighBeam`), handbrake, and cruise control.

### 3. Dirty-Checking & Change Detection (`src/change_detection.rs`)
Game HUDs traditionally waste significant CPU and GPU cycles repainting static UI elements. `minimal-hud-rs` uses a multi-tier dirty-checking engine:

1. **Epsilon Threshold Gating**: Values that change by less than $\epsilon$ (e.g. $|\Delta v| < 0.25\%$) are ignored, preventing micro-jitter redraws.
2. **Frequency Throttling / Rate-Limiting**: High-frequency metrics (speed, RPM) update at $30\text{Hz}$, while metabolic vitals (hunger, thirst) update at $1\text{Hz}$.
3. **Change Flags Mask (`DirtyFlags`)**: Fine-grained bitflags track which exact UI element changed (`HEALTH`, `SPEED`, `GEAR`, etc.).
4. **Exponential Smoothing Needle Interpolator**:
   $$v_{\text{display}} = \operatorname{lerp}(v_{\text{display}}, v_{\text{actual}}, 1 - e^{-\lambda \Delta t})$$
   Guarantees framerate-independent, butter-smooth needle movement with automated snap convergence.

#### Benchmark: Render Cycle Savings
In standard 60 FPS gameplay simulations:
- **Naive Render Loop**: 600 draw calls over 10 seconds.
- **`minimal-hud-rs` Engine**: Only 8–12 draw calls dispatched.
- **Efficiency Gain**: **$>95\%$ reduction in redundant draw calls**!

### 4. Alert & Animation Logic (`src/pulse.rs`)
- Procedural wave generation (`Sine`, `Square`, `Triangle`, `Sawtooth`, and double-peak `Heartbeat`).
- Dynamic urgency scaling: as health or oxygen approaches 0, oscillation frequency scales smoothly from $1.0\text{Hz}$ up to $4.0\text{Hz}$.
- Edge-triggered audio cues:
  - `SeatbeltChime`
  - `HeartbeatThud`
  - `WarningBeep`
  - `LowFuelBeep`
  - `DrowningGasp`
  - `StarvationRumble`

### 5. Bevy ECS Integration (`src/bevy_adapter.rs`)
Includes first-class Bevy 0.15 ECS support via the optional `bevy` feature flag:
- Components: `HudVitals`, `VehicleDashboard`, `HudDirtyState`, `HudAnimationState`.
- Zero-cost system queries utilizing Bevy's native change detection:
  ```rust
  pub fn hud_combined_change_system(
      mut query: Query<CombinedHudQueryItem, Or<(Changed<HudVitals>, Changed<VehicleDashboard>)>>,
  )
  ```
  Entities that are stationary or whose vitals have not changed bypass all processing.

---

## 📦 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
minimal-hud-rs = { version = "0.1.0", features = ["bevy"] }
```

### Basic Rust Usage

```rust
use minimal_hud_rs::prelude::*;

fn main() {
    let mut tracker = HudTracker::new(ChangeDetectionConfig::default());
    let mut vitals = PlayerVitals::default();

    // Damage player by 30 (absorbed by armor)
    vitals.apply_damage(30.0);

    // Evaluate dirty status (returns DirtyFlags::ARMOR)
    let dirty = tracker.evaluate_vitals(&vitals, 1.0 / 60.0);
    if dirty.contains(DirtyFlags::ARMOR) {
        println!("Redraw armor meter: {}%", vitals.armor.percentage());
    }

    tracker.clear_dirty();
}
```

### Bevy ECS Usage

```rust
use bevy::prelude::*;
use minimal_hud_rs::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<HudAudioEvent>()
        .add_systems(Update, (
            hud_vitals_system,
            hud_vehicle_system,
            hud_audio_trigger_system,
        ))
        .run();
}
```

---

## 🧪 Testing

Run the full test suite verifying 100% pass rate:

```bash
cargo test --all-features
cargo test --no-default-features
cargo clippy --all-features
```

---

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
