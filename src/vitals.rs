//! Vital stat models and alert threshold evaluations.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

/// Clamped vital stat representation (typically 0.0 to 100.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VitalStat {
    current: f32,
    max: f32,
}

impl Default for VitalStat {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

impl VitalStat {
    /// Create a new stat clamped between 0.0 and `max`.
    pub fn new(current: f32, max: f32) -> Self {
        let max = max.max(0.0001);
        Self {
            current: current.clamp(0.0, max),
            max,
        }
    }

    /// Create a standard 0..=100 stat.
    pub fn standard(current: f32) -> Self {
        Self::new(current, 100.0)
    }

    /// Get current raw value.
    #[inline]
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Get max raw value.
    #[inline]
    pub fn max(&self) -> f32 {
        self.max
    }

    /// Set current value, clamped to [0, max].
    pub fn set(&mut self, val: f32) {
        self.current = val.clamp(0.0, self.max);
    }

    /// Modify current value by delta (can be positive or negative).
    pub fn modify(&mut self, delta: f32) {
        self.set(self.current + delta);
    }

    /// Ratio from 0.0 to 1.0.
    #[inline]
    pub fn ratio(&self) -> f32 {
        (self.current / self.max).clamp(0.0, 1.0)
    }

    /// Percentage from 0.0 to 100.0.
    #[inline]
    pub fn percentage(&self) -> f32 {
        self.ratio() * 100.0
    }

    /// Whether the stat is depleted (0.0).
    #[inline]
    pub fn is_depleted(&self) -> bool {
        self.current <= 0.0001
    }

    /// Whether the stat is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        (self.current - self.max).abs() <= 0.0001
    }
}

/// Voice proximity broadcast levels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum VoiceProximityMode {
    /// Whisper mode: 1.5 meters radius.
    Whisper,
    /// Normal speaking mode: 5.0 meters radius.
    #[default]
    Normal,
    /// Shout mode: 12.0 meters radius.
    Shout,
    /// Custom radius in meters.
    Custom(f32),
}

impl VoiceProximityMode {
    /// Returns the effective distance in meters.
    pub fn distance_meters(&self) -> f32 {
        match self {
            Self::Whisper => 1.5,
            Self::Normal => 5.0,
            Self::Shout => 12.0,
            Self::Custom(dist) => (*dist).max(0.0),
        }
    }

    /// Cycle to the next standard voice range (Whisper -> Normal -> Shout -> Whisper).
    pub fn cycle_next(&self) -> Self {
        match self {
            Self::Whisper => Self::Normal,
            Self::Normal => Self::Shout,
            Self::Shout | Self::Custom(_) => Self::Whisper,
        }
    }

    /// Display string for HUD badge.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Whisper => "Whisper",
            Self::Normal => "Normal",
            Self::Shout => "Shout",
            Self::Custom(_) => "Custom",
        }
    }
}

/// Player voice communication status.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct VoiceState {
    pub mode: VoiceProximityMode,
    pub is_talking: bool,
    pub radio_channel: Option<u16>,
}

impl VoiceState {
    pub fn new(mode: VoiceProximityMode, is_talking: bool) -> Self {
        Self {
            mode,
            is_talking,
            radio_channel: None,
        }
    }

    pub fn with_radio(mut self, channel: u16) -> Self {
        self.radio_channel = Some(channel);
        self
    }
}

/// Configurable threshold values for dynamic HUD warnings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VitalThresholds {
    /// Low health warning threshold (<= 20.0%).
    pub low_health: f32,
    /// Drowning warning threshold (<= 15.0% oxygen).
    pub drowning: f32,
    /// Dehydration warning threshold (<= 10.0% thirst).
    pub dehydration: f32,
    /// Starvation warning threshold (<= 10.0% hunger).
    pub starvation: f32,
    /// Exhaustion warning threshold (<= 5.0% stamina).
    pub exhaustion: f32,
    /// High stress threshold (>= 80.0% stress).
    pub high_stress: f32,
}

impl Default for VitalThresholds {
    fn default() -> Self {
        Self {
            low_health: 20.0,
            drowning: 15.0,
            dehydration: 10.0,
            starvation: 10.0,
            exhaustion: 5.0,
            high_stress: 80.0,
        }
    }
}

bitflags! {
    /// Active vital alert states.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct VitalAlerts: u32 {
        const NONE         = 0;
        const LOW_HEALTH   = 1 << 0;
        const DROWNING     = 1 << 1;
        const DEHYDRATION  = 1 << 2;
        const STARVATION   = 1 << 3;
        const EXHAUSTION   = 1 << 4;
        const HIGH_STRESS  = 1 << 5;
    }
}

/// Complete player vital stats container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerVitals {
    pub health: VitalStat,
    pub armor: VitalStat,
    pub stamina: VitalStat,
    pub hunger: VitalStat,
    pub thirst: VitalStat,
    pub stress: VitalStat,
    pub oxygen: VitalStat,
    pub voice: VoiceState,
    pub thresholds: VitalThresholds,
}

impl Default for PlayerVitals {
    fn default() -> Self {
        Self {
            health: VitalStat::standard(100.0),
            armor: VitalStat::standard(0.0), // Start with 0 armor
            stamina: VitalStat::standard(100.0),
            hunger: VitalStat::standard(100.0),
            thirst: VitalStat::standard(100.0),
            stress: VitalStat::standard(0.0), // Start with 0 stress
            oxygen: VitalStat::standard(100.0),
            voice: VoiceState::default(),
            thresholds: VitalThresholds::default(),
        }
    }
}

impl PlayerVitals {
    /// Evaluate all active alert flags according to defined thresholds.
    pub fn active_alerts(&self) -> VitalAlerts {
        let mut alerts = VitalAlerts::NONE;

        let eps = 1e-4;

        if self.health.percentage() <= self.thresholds.low_health + eps {
            alerts |= VitalAlerts::LOW_HEALTH;
        }
        if self.oxygen.percentage() <= self.thresholds.drowning + eps {
            alerts |= VitalAlerts::DROWNING;
        }
        if self.thirst.percentage() <= self.thresholds.dehydration + eps {
            alerts |= VitalAlerts::DEHYDRATION;
        }
        if self.hunger.percentage() <= self.thresholds.starvation + eps {
            alerts |= VitalAlerts::STARVATION;
        }
        if self.stamina.percentage() <= self.thresholds.exhaustion + eps {
            alerts |= VitalAlerts::EXHAUSTION;
        }
        if self.stress.percentage() >= self.thresholds.high_stress - eps {
            alerts |= VitalAlerts::HIGH_STRESS;
        }

        alerts
    }

    /// Check if a specific alert is currently active.
    #[inline]
    pub fn has_alert(&self, alert: VitalAlerts) -> bool {
        self.active_alerts().contains(alert)
    }

    /// Apply damage taking armor into account.
    /// Armor absorbs damage at 1:1 up to armor limit; remainder spills to health.
    pub fn apply_damage(&mut self, amount: f32) {
        let damage = amount.max(0.0);
        let armor_val = self.armor.current();

        if armor_val > 0.0 {
            if armor_val >= damage {
                self.armor.modify(-damage);
                return;
            } else {
                let spillover = damage - armor_val;
                self.armor.set(0.0);
                self.health.modify(-spillover);
                return;
            }
        }

        self.health.modify(-damage);
    }

    /// Consume food to replenish hunger.
    pub fn eat(&mut self, amount: f32) {
        self.hunger.modify(amount.max(0.0));
    }

    /// Drink water/beverage to replenish thirst.
    pub fn drink(&mut self, amount: f32) {
        self.thirst.modify(amount.max(0.0));
    }

    /// Heal player health directly.
    pub fn heal(&mut self, amount: f32) {
        self.health.modify(amount.max(0.0));
    }

    /// Restore armor directly.
    pub fn add_armor(&mut self, amount: f32) {
        self.armor.modify(amount.max(0.0));
    }
}
