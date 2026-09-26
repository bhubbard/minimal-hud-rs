//! Error types for minimal-hud-rs.

use thiserror::Error;

/// Top-level error for the HUD library.
#[derive(Debug, Error, PartialEq)]
pub enum HudError {
    #[error("Vital stat error: {0}")]
    Vital(#[from] VitalError),

    #[error("Vehicle telemetry error: {0}")]
    Vehicle(#[from] VehicleError),
}

/// Errors related to vital stats manipulation.
#[derive(Debug, Error, PartialEq)]
pub enum VitalError {
    #[error("Invalid percentage value {0}; expected between 0.0 and 100.0")]
    OutOfRange(f32),

    #[error("Invalid proximity distance {0}m")]
    InvalidVoiceDistance(f32),
}

/// Errors related to vehicle telemetry.
#[derive(Debug, Error, PartialEq)]
pub enum VehicleError {
    #[error("Invalid gear number {0}; valid gears are -1 (Reverse), 0 (Neutral), 1..=7 (Forward)")]
    InvalidGear(i8),

    #[error("Invalid normalized RPM {0}; expected between 0.0 and 1.0")]
    InvalidRpm(f32),

    #[error("Invalid speed scalar {0}")]
    InvalidSpeed(f32),
}
