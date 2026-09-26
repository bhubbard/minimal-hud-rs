use minimal_hud_rs::prelude::*;

#[test]
fn test_vital_stat_clamping_and_ratios() {
    let mut stat = VitalStat::standard(50.0);
    assert_eq!(stat.current(), 50.0);
    assert_eq!(stat.ratio(), 0.5);
    assert_eq!(stat.percentage(), 50.0);

    // Test upper clamp
    stat.set(150.0);
    assert_eq!(stat.current(), 100.0);
    assert!(stat.is_full());

    // Test lower clamp
    stat.set(-25.0);
    assert_eq!(stat.current(), 0.0);
    assert!(stat.is_depleted());

    // Test delta modify
    stat.modify(35.0);
    assert_eq!(stat.current(), 35.0);
}

#[test]
fn test_damage_and_armor_absorption() {
    let mut vitals = PlayerVitals::default();
    vitals.health.set(100.0);
    vitals.armor.set(50.0);

    // Damage absorbed entirely by armor
    vitals.apply_damage(30.0);
    assert_eq!(vitals.armor.current(), 20.0);
    assert_eq!(vitals.health.current(), 100.0);

    // Damage depletes armor and spills over to health
    vitals.apply_damage(40.0);
    assert_eq!(vitals.armor.current(), 0.0);
    assert_eq!(vitals.health.current(), 80.0);

    // Further damage goes straight to health
    vitals.apply_damage(25.0);
    assert_eq!(vitals.health.current(), 55.0);
}

#[test]
fn test_dynamic_threshold_alerts() {
    let mut vitals = PlayerVitals::default();

    // Default healthy state has no alerts
    assert_eq!(vitals.active_alerts(), VitalAlerts::NONE);

    // Low health alert <= 20%
    vitals.health.set(20.0);
    assert!(vitals.has_alert(VitalAlerts::LOW_HEALTH));
    vitals.health.set(20.1);
    assert!(!vitals.has_alert(VitalAlerts::LOW_HEALTH));

    // Drowning alert <= 15% oxygen
    vitals.oxygen.set(15.0);
    assert!(vitals.has_alert(VitalAlerts::DROWNING));

    // Dehydration alert <= 10% thirst
    vitals.thirst.set(10.0);
    assert!(vitals.has_alert(VitalAlerts::DEHYDRATION));

    // Starvation alert <= 10% hunger
    vitals.hunger.set(10.0);
    assert!(vitals.has_alert(VitalAlerts::STARVATION));

    // Exhaustion alert <= 5% stamina
    vitals.stamina.set(5.0);
    assert!(vitals.has_alert(VitalAlerts::EXHAUSTION));

    // High stress alert >= 80%
    vitals.stress.set(80.0);
    assert!(vitals.has_alert(VitalAlerts::HIGH_STRESS));

    // Multiple alerts active simultaneously
    let alerts = vitals.active_alerts();
    assert!(
        alerts.contains(VitalAlerts::DROWNING | VitalAlerts::DEHYDRATION | VitalAlerts::STARVATION)
    );
}

#[test]
fn test_voice_proximity_modes() {
    let mut voice = VoiceState::new(VoiceProximityMode::Whisper, false);
    assert_eq!(voice.mode.distance_meters(), 1.5);
    assert_eq!(voice.mode.label(), "Whisper");

    voice.mode = voice.mode.cycle_next();
    assert_eq!(voice.mode, VoiceProximityMode::Normal);
    assert_eq!(voice.mode.distance_meters(), 5.0);
    assert_eq!(voice.mode.label(), "Normal");

    voice.mode = voice.mode.cycle_next();
    assert_eq!(voice.mode, VoiceProximityMode::Shout);
    assert_eq!(voice.mode.distance_meters(), 12.0);
    assert_eq!(voice.mode.label(), "Shout");

    voice.mode = voice.mode.cycle_next();
    assert_eq!(voice.mode, VoiceProximityMode::Whisper);

    // Custom mode
    let custom = VoiceProximityMode::Custom(25.0);
    assert_eq!(custom.distance_meters(), 25.0);
    assert_eq!(custom.cycle_next(), VoiceProximityMode::Whisper);
}
