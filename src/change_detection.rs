//! Dirty-checking, rate-limiting, and exponential smoothing for HUD elements.

use crate::vehicle::VehicleTelemetry;
use crate::vitals::PlayerVitals;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Mask tracking which HUD elements require redrawing.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
    pub struct DirtyFlags: u32 {
        const NONE           = 0;

        // Vital stats
        const HEALTH         = 1 << 0;
        const ARMOR          = 1 << 1;
        const STAMINA        = 1 << 2;
        const HUNGER         = 1 << 3;
        const THIRST         = 1 << 4;
        const STRESS         = 1 << 5;
        const OXYGEN         = 1 << 6;
        const VOICE          = 1 << 7;

        // Vehicle telemetry
        const SPEED          = 1 << 8;
        const RPM            = 1 << 9;
        const GEAR           = 1 << 10;
        const FUEL           = 1 << 11;
        const ENGINE         = 1 << 12;
        const SEATBELT       = 1 << 13;
        const LIGHTS         = 1 << 14;
        const HANDBRAKE      = 1 << 15;
        const CRUISE_CONTROL = 1 << 16;

        // Convenience groups
        const ALL_VITALS = Self::HEALTH.bits()
            | Self::ARMOR.bits()
            | Self::STAMINA.bits()
            | Self::HUNGER.bits()
            | Self::THIRST.bits()
            | Self::STRESS.bits()
            | Self::OXYGEN.bits()
            | Self::VOICE.bits();

        const ALL_VEHICLE = Self::SPEED.bits()
            | Self::RPM.bits()
            | Self::GEAR.bits()
            | Self::FUEL.bits()
            | Self::ENGINE.bits()
            | Self::SEATBELT.bits()
            | Self::LIGHTS.bits()
            | Self::HANDBRAKE.bits()
            | Self::CRUISE_CONTROL.bits();

        const ALL = Self::ALL_VITALS.bits() | Self::ALL_VEHICLE.bits();
    }
}

/// Exponential smoothing interpolator:
///
/// $$v_{\text{display}} = \operatorname{lerp}(v_{\text{display}}, v_{\text{actual}}, 1 - e^{-\lambda \Delta t})$$
///
/// This provides framerate-independent, jitter-free visual needle/meter movement.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExpSmoother {
    current: f32,
    /// Decay rate lambda ($\lambda \ge 0$). Higher values snap faster; typical values 8.0 - 20.0.
    lambda: f32,
}

impl Default for ExpSmoother {
    fn default() -> Self {
        Self {
            current: 0.0,
            lambda: 12.0,
        }
    }
}

impl ExpSmoother {
    pub fn new(initial: f32, lambda: f32) -> Self {
        Self {
            current: initial,
            lambda: lambda.max(0.1),
        }
    }

    #[inline]
    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn set_immediate(&mut self, val: f32) {
        self.current = val;
    }

    /// Advance smoother toward target value given time elapsed `dt` in seconds.
    pub fn update(&mut self, target: f32, dt_seconds: f32) -> f32 {
        if dt_seconds <= 0.0 {
            return self.current;
        }

        // Snap if close to prevent asymptotic micro-oscillations
        if (self.current - target).abs() < 0.005 {
            self.current = target;
            return self.current;
        }

        // factor = 1 - exp(-lambda * dt)
        let factor = 1.0 - (-self.lambda * dt_seconds).exp();
        self.current += (target - self.current) * factor;
        self.current
    }
}

/// Configuration options for change detection and throttle frequencies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeDetectionConfig {
    /// Epsilon difference threshold for float vitals (default: 0.25%).
    pub vitals_epsilon: f32,
    /// Epsilon difference threshold for vehicle speed in mph (default: 0.5 mph).
    pub speed_epsilon: f32,
    /// Epsilon difference threshold for vehicle RPM (default: 0.02).
    pub rpm_epsilon: f32,
    /// Epsilon difference threshold for fuel (default: 0.25%).
    pub fuel_epsilon: f32,
    /// Throttle interval for slow vital updates in seconds (default: 1.0s / 1 Hz).
    pub vitals_throttle_seconds: f32,
    /// Throttle interval for fast vehicle telemetry in seconds (default: 0.0333s / ~30 Hz).
    pub vehicle_throttle_seconds: f32,
}

impl Default for ChangeDetectionConfig {
    fn default() -> Self {
        Self {
            vitals_epsilon: 0.25,
            speed_epsilon: 0.5,
            rpm_epsilon: 0.02,
            fuel_epsilon: 0.25,
            vitals_throttle_seconds: 1.0,
            vehicle_throttle_seconds: 0.0333,
        }
    }
}

/// Change detection engine that compares incoming states against last flushed states,
/// applies rate-limiting throttles and epsilon filtering, and accumulates `DirtyFlags`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudTracker {
    pub config: ChangeDetectionConfig,
    last_vitals: Option<PlayerVitals>,
    last_vehicle: Option<VehicleTelemetry>,
    vitals_timer: f32,
    vehicle_timer: f32,
    active_dirty: DirtyFlags,

    // Performance profiling metrics
    total_evaluations: u64,
    dispatched_renders: u64,
}

impl Default for HudTracker {
    fn default() -> Self {
        Self::new(ChangeDetectionConfig::default())
    }
}

impl HudTracker {
    pub fn new(config: ChangeDetectionConfig) -> Self {
        Self {
            config,
            last_vitals: None,
            last_vehicle: None,
            vitals_timer: 0.0,
            vehicle_timer: 0.0,
            active_dirty: DirtyFlags::NONE,
            total_evaluations: 0,
            dispatched_renders: 0,
        }
    }

    /// Read currently accumulated dirty flags.
    #[inline]
    pub fn dirty_flags(&self) -> DirtyFlags {
        self.active_dirty
    }

    /// Whether any UI elements need redraw.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        !self.active_dirty.is_empty()
    }

    /// Reset dirty flags after UI has completed a render pass.
    pub fn clear_dirty(&mut self) {
        self.active_dirty = DirtyFlags::NONE;
    }

    /// Force mark all elements dirty (e.g. on full UI reload or initial boot).
    pub fn force_dirty_all(&mut self) {
        self.active_dirty |= DirtyFlags::ALL;
        self.dispatched_renders += 1;
    }

    /// Total evaluation cycles processed.
    #[inline]
    pub fn total_evaluations(&self) -> u64 {
        self.total_evaluations
    }

    /// Total times a UI redraw was required.
    #[inline]
    pub fn dispatched_renders(&self) -> u64 {
        self.dispatched_renders
    }

    /// Calculate percentage of redundant render frames saved by dirty-checking.
    pub fn render_savings_percentage(&self) -> f32 {
        if self.total_evaluations == 0 {
            return 0.0;
        }
        let saved = self
            .total_evaluations
            .saturating_sub(self.dispatched_renders);
        (saved as f32 / self.total_evaluations as f32) * 100.0
    }

    /// Evaluate player vitals and accumulate dirty flags.
    pub fn evaluate_vitals(&mut self, current: &PlayerVitals, dt_seconds: f32) -> DirtyFlags {
        self.total_evaluations += 1;
        self.vitals_timer += dt_seconds;

        // If never evaluated before, mark all vitals dirty immediately
        let Some(prev) = &self.last_vitals else {
            self.last_vitals = Some(current.clone());
            let flags = DirtyFlags::ALL_VITALS;
            self.active_dirty |= flags;
            self.dispatched_renders += 1;
            return flags;
        };

        // If throttled, skip dirty checking until interval elapsed
        if self.vitals_timer < self.config.vitals_throttle_seconds {
            return DirtyFlags::NONE;
        }

        self.vitals_timer = 0.0;
        let mut flags = DirtyFlags::NONE;
        let eps = self.config.vitals_epsilon;

        if (current.health.percentage() - prev.health.percentage()).abs() >= eps {
            flags |= DirtyFlags::HEALTH;
        }
        if (current.armor.percentage() - prev.armor.percentage()).abs() >= eps {
            flags |= DirtyFlags::ARMOR;
        }
        if (current.stamina.percentage() - prev.stamina.percentage()).abs() >= eps {
            flags |= DirtyFlags::STAMINA;
        }
        if (current.hunger.percentage() - prev.hunger.percentage()).abs() >= eps {
            flags |= DirtyFlags::HUNGER;
        }
        if (current.thirst.percentage() - prev.thirst.percentage()).abs() >= eps {
            flags |= DirtyFlags::THIRST;
        }
        if (current.stress.percentage() - prev.stress.percentage()).abs() >= eps {
            flags |= DirtyFlags::STRESS;
        }
        if (current.oxygen.percentage() - prev.oxygen.percentage()).abs() >= eps {
            flags |= DirtyFlags::OXYGEN;
        }
        if current.voice != prev.voice {
            flags |= DirtyFlags::VOICE;
        }

        if !flags.is_empty() {
            self.last_vitals = Some(current.clone());
            self.active_dirty |= flags;
            self.dispatched_renders += 1;
        }

        flags
    }

    /// Evaluate vehicle telemetry and accumulate dirty flags.
    pub fn evaluate_vehicle(&mut self, current: &VehicleTelemetry, dt_seconds: f32) -> DirtyFlags {
        self.total_evaluations += 1;
        self.vehicle_timer += dt_seconds;

        let Some(prev) = &self.last_vehicle else {
            self.last_vehicle = Some(current.clone());
            let flags = DirtyFlags::ALL_VEHICLE;
            self.active_dirty |= flags;
            self.dispatched_renders += 1;
            return flags;
        };

        if self.vehicle_timer < self.config.vehicle_throttle_seconds {
            return DirtyFlags::NONE;
        }

        self.vehicle_timer = 0.0;
        let mut flags = DirtyFlags::NONE;

        if (current.speed.as_mph() - prev.speed.as_mph()).abs() >= self.config.speed_epsilon {
            flags |= DirtyFlags::SPEED;
        }
        if (current.rpm - prev.rpm).abs() >= self.config.rpm_epsilon {
            flags |= DirtyFlags::RPM;
        }
        if current.gear != prev.gear {
            flags |= DirtyFlags::GEAR;
        }
        if (current.fuel - prev.fuel).abs() >= self.config.fuel_epsilon {
            flags |= DirtyFlags::FUEL;
        }
        if (current.engine.current() - prev.engine.current()).abs() >= 1.0 {
            flags |= DirtyFlags::ENGINE;
        }
        if current.seatbelt != prev.seatbelt {
            flags |= DirtyFlags::SEATBELT;
        }
        if current.lights != prev.lights {
            flags |= DirtyFlags::LIGHTS;
        }
        if current.handbrake != prev.handbrake {
            flags |= DirtyFlags::HANDBRAKE;
        }
        if current.cruise_control != prev.cruise_control {
            flags |= DirtyFlags::CRUISE_CONTROL;
        }

        if !flags.is_empty() {
            self.last_vehicle = Some(current.clone());
            self.active_dirty |= flags;
            self.dispatched_renders += 1;
        }

        flags
    }
}
