//! Vehicle telemetry models, conversions, and status tracking.

use crate::error::VehicleError;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

/// Conversion constant: 1 mph in km/h.
pub const MPH_TO_KMH: f32 = 1.609344;
/// Conversion constant: 1 km/h in mph.
pub const KMH_TO_MPH: f32 = 0.621_371_2;
/// Conversion constant: 1 m/s in km/h.
pub const MPS_TO_KMH: f32 = 3.6;
/// Conversion constant: 1 m/s in mph.
pub const MPS_TO_MPH: f32 = 2.236_936_3;

/// Units of speed measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SpeedUnit {
    #[default]
    Mph,
    Kmh,
}

/// Strongly typed scalar vehicle speed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Speed {
    /// Internal speed stored in meters per second (SI unit).
    mps: f32,
}

impl Speed {
    /// Create speed from meters per second.
    pub fn from_mps(mps: f32) -> Self {
        Self { mps: mps.max(0.0) }
    }

    /// Create speed from kilometers per hour.
    pub fn from_kmh(kmh: f32) -> Self {
        Self {
            mps: (kmh.max(0.0) / MPS_TO_KMH),
        }
    }

    /// Create speed from miles per hour.
    pub fn from_mph(mph: f32) -> Self {
        Self {
            mps: (mph.max(0.0) / MPS_TO_MPH),
        }
    }

    /// Speed value in meters per second.
    #[inline]
    pub fn as_mps(&self) -> f32 {
        self.mps
    }

    /// Speed value in kilometers per hour.
    #[inline]
    pub fn as_kmh(&self) -> f32 {
        self.mps * MPS_TO_KMH
    }

    /// Speed value in miles per hour.
    #[inline]
    pub fn as_mph(&self) -> f32 {
        self.mps * MPS_TO_MPH
    }

    /// Speed value in the specified unit.
    pub fn as_unit(&self, unit: SpeedUnit) -> f32 {
        match unit {
            SpeedUnit::Mph => self.as_mph(),
            SpeedUnit::Kmh => self.as_kmh(),
        }
    }

    /// Formats rounded integer speed string for HUD display.
    pub fn display_value(&self, unit: SpeedUnit) -> u32 {
        self.as_unit(unit).round() as u32
    }
}

/// Transmission gear selector state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Gear {
    Reverse,
    #[default]
    Neutral,
    Forward(u8),
}

impl Gear {
    /// Create gear from integer (-1 = Reverse, 0 = Neutral, 1..=7 = Forward).
    pub fn from_i8(val: i8) -> Result<Self, VehicleError> {
        match val {
            -1 => Ok(Self::Reverse),
            0 => Ok(Self::Neutral),
            1..=7 => Ok(Self::Forward(val as u8)),
            other => Err(VehicleError::InvalidGear(other)),
        }
    }

    /// Convert gear to integer representation.
    pub fn as_i8(&self) -> i8 {
        match self {
            Self::Reverse => -1,
            Self::Neutral => 0,
            Self::Forward(g) => *g as i8,
        }
    }

    /// Get short string representation for HUD rendering ("R", "N", "1".."7").
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::Reverse => "R",
            Self::Neutral => "N",
            Self::Forward(1) => "1",
            Self::Forward(2) => "2",
            Self::Forward(3) => "3",
            Self::Forward(4) => "4",
            Self::Forward(5) => "5",
            Self::Forward(6) => "6",
            Self::Forward(7) => "7",
            Self::Forward(_) => "?",
        }
    }
}

/// Vehicle exterior light status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LightState {
    #[default]
    Off,
    LowBeam,
    HighBeam,
}

impl LightState {
    /// Whether any headlights are active.
    #[inline]
    pub fn is_on(&self) -> bool {
        !matches!(self, Self::Off)
    }

    /// Cycle headlights: Off -> LowBeam -> HighBeam -> Off.
    pub fn cycle_next(&self) -> Self {
        match self {
            Self::Off => Self::LowBeam,
            Self::LowBeam => Self::HighBeam,
            Self::HighBeam => Self::Off,
        }
    }
}

/// Engine damage categorization based on GTA/FiveM 0..=1000 scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineState {
    /// Engine is in pristine condition (> 800).
    Optimal,
    /// Minor wear or superficial damage (600..=800).
    MinorDamage,
    /// Moderate damage, reduced acceleration (300..=600).
    ModerateDamage,
    /// Critical damage, engine smoking and prone to stalling (0..=300).
    Critical,
    /// Engine completely broken down (<= 0).
    Destroyed,
}

/// Engine health tracker (standard 0.0 to 1000.0 health scale).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EngineHealth {
    current: f32,
    max: f32,
}

impl Default for EngineHealth {
    fn default() -> Self {
        Self {
            current: 1000.0,
            max: 1000.0,
        }
    }
}

impl EngineHealth {
    pub fn new(current: f32) -> Self {
        Self {
            current: current.clamp(0.0, 1000.0),
            max: 1000.0,
        }
    }

    #[inline]
    pub fn current(&self) -> f32 {
        self.current
    }

    #[inline]
    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn set(&mut self, val: f32) {
        self.current = val.clamp(0.0, self.max);
    }

    pub fn modify(&mut self, delta: f32) {
        self.set(self.current + delta);
    }

    /// Percentage from 0.0 to 100.0%.
    #[inline]
    pub fn percentage(&self) -> f32 {
        (self.current / self.max) * 100.0
    }

    /// Qualitative engine operating condition.
    pub fn state(&self) -> EngineState {
        if self.current <= 0.0 {
            EngineState::Destroyed
        } else if self.current <= 300.0 {
            EngineState::Critical
        } else if self.current <= 600.0 {
            EngineState::ModerateDamage
        } else if self.current <= 800.0 {
            EngineState::MinorDamage
        } else {
            EngineState::Optimal
        }
    }

    #[inline]
    pub fn is_critical(&self) -> bool {
        self.current <= 300.0
    }
}

bitflags! {
    /// Active vehicle warning indicators.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct VehicleAlerts: u32 {
        const NONE               = 0;
        const UNBUCKLED_SEATBELT = 1 << 0;
        const LOW_FUEL           = 1 << 1;
        const ENGINE_CRITICAL    = 1 << 2;
        const HANDBRAKE_ENGAGED  = 1 << 3;
    }
}

/// Comprehensive vehicle telemetry state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleTelemetry {
    pub speed: Speed,
    /// Normalized engine RPM (0.0 to 1.0).
    pub rpm: f32,
    pub gear: Gear,
    /// Fuel level percentage (0.0 to 100.0).
    pub fuel: f32,
    pub engine: EngineHealth,
    pub seatbelt: bool,
    pub lights: LightState,
    pub handbrake: bool,
    pub cruise_control: bool,
    /// Threshold below which low fuel alert triggers (default: 15.0%).
    pub low_fuel_threshold: f32,
    /// RPM threshold considered redline (default: 0.85).
    pub redline_threshold: f32,
}

impl Default for VehicleTelemetry {
    fn default() -> Self {
        Self {
            speed: Speed::default(),
            rpm: 0.0,
            gear: Gear::Neutral,
            fuel: 100.0,
            engine: EngineHealth::default(),
            seatbelt: false,
            lights: LightState::Off,
            handbrake: true,
            cruise_control: false,
            low_fuel_threshold: 15.0,
            redline_threshold: 0.85,
        }
    }
}

impl VehicleTelemetry {
    /// Evaluate all active vehicle alerts.
    pub fn active_alerts(&self) -> VehicleAlerts {
        let mut alerts = VehicleAlerts::NONE;

        // Seatbelt warning if unbuckled while moving (> 2.0 mph)
        if !self.seatbelt && self.speed.as_mph() > 2.0 {
            alerts |= VehicleAlerts::UNBUCKLED_SEATBELT;
        }

        // Low fuel warning
        if self.fuel <= self.low_fuel_threshold {
            alerts |= VehicleAlerts::LOW_FUEL;
        }

        // Critical engine warning
        if self.engine.is_critical() {
            alerts |= VehicleAlerts::ENGINE_CRITICAL;
        }

        // Handbrake warning if engaged while trying to move
        if self.handbrake {
            alerts |= VehicleAlerts::HANDBRAKE_ENGAGED;
        }

        alerts
    }

    /// Check if engine is currently at or above redline.
    #[inline]
    pub fn is_redlining(&self) -> bool {
        self.rpm >= self.redline_threshold
    }

    /// Set RPM with clamping to [0.0, 1.0].
    pub fn set_rpm(&mut self, rpm: f32) {
        self.rpm = rpm.clamp(0.0, 1.0);
    }

    /// Set fuel with clamping to [0.0, 100.0].
    pub fn set_fuel(&mut self, fuel: f32) {
        self.fuel = fuel.clamp(0.0, 100.0);
    }
}
