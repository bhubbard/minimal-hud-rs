//! Bevy ECS integration module providing components, change detection queries, and systems.

use crate::change_detection::{DirtyFlags, HudTracker};
use crate::pulse::{AudioCue, AudioTriggerManager};
use crate::vehicle::VehicleTelemetry;
use crate::vitals::PlayerVitals;

use bevy_ecs::component::Component;
use bevy_ecs::event::{Event, EventWriter};
use bevy_ecs::prelude::*;

/// Bevy Component wrapping player vital statistics.
#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct HudVitals(pub PlayerVitals);

/// Bevy Component wrapping vehicle dashboard telemetry.
#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct VehicleDashboard(pub VehicleTelemetry);

/// Bevy Component tracking dirty-checking state and render flags.
#[derive(Component, Debug, Clone, Default)]
pub struct HudDirtyState {
    pub tracker: HudTracker,
    pub flags: DirtyFlags,
}

impl HudDirtyState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Bevy Component managing procedural pulsing animations and audio state.
#[derive(Component, Debug, Clone, Default)]
pub struct HudAnimationState {
    pub audio_manager: AudioTriggerManager,
}

/// Bevy Event emitted when a HUD audio cue triggers.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudAudioEvent(pub AudioCue);

/// Bevy system that inspects entities whose vitals changed via native `Changed<HudVitals>`.
/// Employs fine-grained dirty flags and throttling to eliminate redundant UI recalculations.
pub fn hud_vitals_system(mut query: Query<(&HudVitals, &mut HudDirtyState), Changed<HudVitals>>) {
    for (vitals, mut state) in query.iter_mut() {
        // Delta time assumed at 1/60s if frame time not injected directly
        let flags = state.tracker.evaluate_vitals(&vitals.0, 1.0 / 60.0);
        state.flags |= flags;
    }
}

/// Bevy system that inspects entities whose vehicle telemetry changed via `Changed<VehicleDashboard>`.
pub fn hud_vehicle_system(
    mut query: Query<(&VehicleDashboard, &mut HudDirtyState), Changed<VehicleDashboard>>,
) {
    for (vehicle, mut state) in query.iter_mut() {
        let flags = state.tracker.evaluate_vehicle(&vehicle.0, 1.0 / 60.0);
        state.flags |= flags;
    }
}

pub type CombinedHudQueryItem<'a> = (
    Option<&'a HudVitals>,
    Option<&'a VehicleDashboard>,
    &'a mut HudDirtyState,
);

pub type CombinedHudFilter = Or<(Changed<HudVitals>, Changed<VehicleDashboard>)>;

/// Bevy combined system using `Or<(Changed<HudVitals>, Changed<VehicleDashboard>)>`
/// guaranteeing ZERO CPU overhead on stationary, idle entities.
#[allow(clippy::type_complexity)]
pub fn hud_combined_change_system(mut query: Query<CombinedHudQueryItem, CombinedHudFilter>) {
    for (vitals, vehicle, mut state) in query.iter_mut() {
        if let Some(v) = vitals {
            let f = state.tracker.evaluate_vitals(&v.0, 1.0 / 60.0);
            state.flags |= f;
        }
        if let Some(veh) = vehicle {
            let f = state.tracker.evaluate_vehicle(&veh.0, 1.0 / 60.0);
            state.flags |= f;
        }
    }
}

/// Bevy system monitoring alerts and emitting `HudAudioEvent` upon state transitions.
pub fn hud_audio_trigger_system(
    mut query: Query<(
        Option<&HudVitals>,
        Option<&VehicleDashboard>,
        &mut HudAnimationState,
    )>,
    mut event_writer: EventWriter<HudAudioEvent>,
) {
    for (vitals, vehicle, mut anim) in query.iter_mut() {
        let cues =
            anim.audio_manager
                .update(vitals.map(|v| &v.0), vehicle.map(|v| &v.0), 1.0 / 60.0);

        for cue in cues {
            event_writer.send(HudAudioEvent(cue));
        }
    }
}
