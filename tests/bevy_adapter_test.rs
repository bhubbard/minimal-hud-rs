#[cfg(feature = "bevy")]
use minimal_hud_rs::prelude::*;

#[cfg(feature = "bevy")]
#[test]
fn test_bevy_vitals_system_and_change_detection() {
    use bevy_ecs::event::Events;
    use bevy_ecs::schedule::Schedule;
    use bevy_ecs::world::World;

    let mut world = World::new();
    world.init_resource::<Events<HudAudioEvent>>();

    let mut schedule = Schedule::default();
    schedule.add_systems(hud_vitals_system);
    schedule.add_systems(hud_vehicle_system);
    schedule.add_systems(hud_audio_trigger_system);

    // Spawn an entity with HudVitals and HudDirtyState
    let mut initial_vitals = PlayerVitals::default();
    initial_vitals.health.set(100.0);

    let entity = world
        .spawn((
            HudVitals(initial_vitals),
            HudDirtyState::default(),
            HudAnimationState::default(),
        ))
        .id();

    // First frame execution initializes the state
    schedule.run(&mut world);

    let dirty_state = world.get::<HudDirtyState>(entity).unwrap();
    assert!(dirty_state.flags.contains(DirtyFlags::ALL_VITALS));

    // Clear dirty flags
    world.get_mut::<HudDirtyState>(entity).unwrap().flags = DirtyFlags::NONE;

    // Run schedule again WITHOUT changing vitals
    schedule.run(&mut world);
    let dirty_state = world.get::<HudDirtyState>(entity).unwrap();
    assert_eq!(
        dirty_state.flags,
        DirtyFlags::NONE,
        "No change detected -> zero overhead"
    );

    // Modify health
    {
        let mut vitals = world.get_mut::<HudVitals>(entity).unwrap();
        vitals.0.health.set(50.0);
    }

    // Now Changed<HudVitals> will trigger the system!
    // Set config throttle to 0 in tracker to allow immediate flag assert
    world
        .get_mut::<HudDirtyState>(entity)
        .unwrap()
        .tracker
        .config
        .vitals_throttle_seconds = 0.0;

    schedule.run(&mut world);
    let dirty_state = world.get::<HudDirtyState>(entity).unwrap();
    assert!(dirty_state.flags.contains(DirtyFlags::HEALTH));
}
