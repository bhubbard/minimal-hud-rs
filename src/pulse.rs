//! Alert animations, low-stat pulsing waveforms, and audio cues.

use crate::vehicle::VehicleTelemetry;
use crate::vitals::{PlayerVitals, VitalAlerts};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Waveform shape for visual icon pulsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PulseWaveform {
    #[default]
    Sine,
    Square,
    Triangle,
    Sawtooth,
    /// Double-beat systolic/diastolic rhythm for heartbeat animations.
    Heartbeat,
}

/// Configuration for pulse wave generator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PulseConfig {
    /// Oscillation frequency in Hertz (cycles per second).
    pub frequency: f32,
    /// Oscillation amplitude (0.0 to 1.0).
    pub amplitude: f32,
    /// Base offset level (minimum brightness/opacity).
    pub offset: f32,
    /// Duty cycle for square/pulse waveforms (0.0 to 1.0).
    pub duty_cycle: f32,
    /// Selected waveform shape.
    pub waveform: PulseWaveform,
}

impl Default for PulseConfig {
    fn default() -> Self {
        Self {
            frequency: 1.5,
            amplitude: 0.5,
            offset: 0.5,
            duty_cycle: 0.5,
            waveform: PulseWaveform::Sine,
        }
    }
}

/// Procedural wave generator for HUD icon pulsing and animations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PulseGenerator {
    pub config: PulseConfig,
    elapsed: f32,
}

impl Default for PulseGenerator {
    fn default() -> Self {
        Self::new(PulseConfig::default())
    }
}

impl PulseGenerator {
    pub fn new(config: PulseConfig) -> Self {
        Self {
            config,
            elapsed: 0.0,
        }
    }

    /// Advance internal clock by delta time in seconds.
    pub fn tick(&mut self, dt_seconds: f32) {
        self.elapsed = (self.elapsed + dt_seconds) % 1000.0;
    }

    /// Sample current pulse output in range [0.0, 1.0].
    pub fn sample_current(&self) -> f32 {
        self.sample_at(self.elapsed)
    }

    /// Sample pulse waveform value at an arbitrary time in seconds.
    pub fn sample_at(&self, t_seconds: f32) -> f32 {
        let freq = self.config.frequency.max(0.01);
        let phase = (t_seconds * freq) % 1.0; // 0.0 to 1.0

        let raw = match self.config.waveform {
            PulseWaveform::Sine => (2.0 * PI * phase).sin() * 0.5 + 0.5,
            PulseWaveform::Square => {
                if phase < self.config.duty_cycle {
                    1.0
                } else {
                    0.0
                }
            }
            PulseWaveform::Triangle => {
                if phase < 0.5 {
                    phase * 2.0
                } else {
                    2.0 - (phase * 2.0)
                }
            }
            PulseWaveform::Sawtooth => phase,
            PulseWaveform::Heartbeat => {
                // Dual peak pulse: first peak at phase ~0.15, second peak at phase ~0.40
                if phase < 0.25 {
                    (PI * phase / 0.25).sin().powi(2)
                } else if (0.28..0.50).contains(&phase) {
                    let sub = (phase - 0.28) / 0.22;
                    (PI * sub).sin().powi(2) * 0.65
                } else {
                    0.0
                }
            }
        };

        (self.config.offset + self.config.amplitude * (raw * 2.0 - 1.0)).clamp(0.0, 1.0)
    }

    /// Dynamically adjust frequency based on a low stat ratio (e.g. Health 0.0 to 0.2).
    /// Closer to 0 increases urgency up to `max_freq`.
    pub fn scale_urgency(
        &mut self,
        current_ratio: f32,
        threshold_ratio: f32,
        min_freq: f32,
        max_freq: f32,
    ) {
        if current_ratio >= threshold_ratio || threshold_ratio <= 0.0 {
            self.config.frequency = min_freq;
            return;
        }

        // urgency factor: 0.0 (at threshold) to 1.0 (at 0)
        let urgency = 1.0 - (current_ratio / threshold_ratio).clamp(0.0, 1.0);
        self.config.frequency = min_freq + (max_freq - min_freq) * urgency;
    }
}

/// Identifiers for distinctive game/HUD audio cues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioCue {
    /// Repeating chime when unbuckled above threshold speed.
    SeatbeltChime,
    /// Deep thudding heartbeat sound when health is critically low.
    HeartbeatThud,
    /// Alert beep when entering a critical threshold (e.g. low fuel, starvation).
    WarningBeep,
    /// Subdued warning chime for low fuel.
    LowFuelBeep,
    /// Underwater gasp for breath when oxygen is critical.
    DrowningGasp,
    /// Stomach rumble when starvation reaches critical.
    StarvationRumble,
}

/// Audio dispatch manager that tracks edge triggers and cooldowns
/// to prevent audio spamming across tick cycles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioTriggerManager {
    prev_alerts: VitalAlerts,
    seatbelt_chime_timer: f32,
    heartbeat_timer: f32,
    low_fuel_triggered: bool,
}

impl Default for AudioTriggerManager {
    fn default() -> Self {
        Self {
            prev_alerts: VitalAlerts::NONE,
            seatbelt_chime_timer: 0.0,
            heartbeat_timer: 0.0,
            low_fuel_triggered: false,
        }
    }
}

impl AudioTriggerManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluate vitals and vehicle states, returning audio cues that should be played this frame.
    pub fn update(
        &mut self,
        vitals: Option<&PlayerVitals>,
        vehicle: Option<&VehicleTelemetry>,
        dt_seconds: f32,
    ) -> Vec<AudioCue> {
        let mut cues = Vec::new();

        // 1. Vitals triggers
        if let Some(v) = vitals {
            let active_alerts = v.active_alerts();
            let newly_triggered = active_alerts & !self.prev_alerts;

            if newly_triggered.contains(VitalAlerts::LOW_HEALTH) {
                cues.push(AudioCue::WarningBeep);
            }
            if newly_triggered.contains(VitalAlerts::DROWNING) {
                cues.push(AudioCue::DrowningGasp);
            }
            if newly_triggered.contains(VitalAlerts::STARVATION) {
                cues.push(AudioCue::StarvationRumble);
            }

            // Periodic heartbeat when health is critically low (<= 20%)
            if active_alerts.contains(VitalAlerts::LOW_HEALTH) {
                self.heartbeat_timer += dt_seconds;
                // Urgency interval: from 1.2s down to 0.4s as health approaches 0
                let health_pct = v.health.percentage().clamp(0.0, 20.0);
                let interval = 0.4 + (health_pct / 20.0) * 0.8;

                if self.heartbeat_timer >= interval {
                    self.heartbeat_timer = 0.0;
                    cues.push(AudioCue::HeartbeatThud);
                }
            } else {
                self.heartbeat_timer = 0.0;
            }

            self.prev_alerts = active_alerts;
        }

        // 2. Vehicle triggers
        if let Some(veh) = vehicle {
            // Seatbelt warning chime: triggers if moving > 5 mph without seatbelt, repeating every 1.5s
            if !veh.seatbelt && veh.speed.as_mph() > 5.0 {
                self.seatbelt_chime_timer += dt_seconds;
                if self.seatbelt_chime_timer >= 1.5 {
                    self.seatbelt_chime_timer = 0.0;
                    cues.push(AudioCue::SeatbeltChime);
                }
            } else {
                self.seatbelt_chime_timer = 1.4; // Primed to ring quickly when starting to drive
            }

            // Low fuel one-shot alert beep
            if veh.fuel <= veh.low_fuel_threshold {
                if !self.low_fuel_triggered {
                    self.low_fuel_triggered = true;
                    cues.push(AudioCue::LowFuelBeep);
                }
            } else {
                self.low_fuel_triggered = false;
            }
        }

        cues
    }
}
