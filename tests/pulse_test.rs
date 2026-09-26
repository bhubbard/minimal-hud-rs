use minimal_hud_rs::prelude::*;

#[test]
fn test_pulse_waveform_sampling() {
    let mut pulse_gen = PulseGenerator::new(PulseConfig {
        frequency: 1.0, // 1 Hz (1 sec period)
        amplitude: 0.5,
        offset: 0.5,
        duty_cycle: 0.5,
        waveform: PulseWaveform::Sine,
    });

    // At t=0.0: sin(0) = 0 -> normalized 0.5
    assert!((pulse_gen.sample_at(0.0) - 0.5).abs() < 0.01);
    // At t=0.25: sin(pi/2) = 1 -> normalized 1.0
    assert!((pulse_gen.sample_at(0.25) - 1.0).abs() < 0.01);
    // At t=0.75: sin(3pi/2) = -1 -> normalized 0.0
    assert!((pulse_gen.sample_at(0.75) - 0.0).abs() < 0.01);

    // Square wave test
    pulse_gen.config.waveform = PulseWaveform::Square;
    assert_eq!(pulse_gen.sample_at(0.1), 1.0);
    assert_eq!(pulse_gen.sample_at(0.8), 0.0);

    // Heartbeat wave test
    pulse_gen.config.waveform = PulseWaveform::Heartbeat;
    let sample = pulse_gen.sample_at(0.125);
    assert!(sample > 0.5);
}

#[test]
fn test_urgency_frequency_scaling() {
    let mut pulse_gen = PulseGenerator::new(PulseConfig::default());

    // Health is 20% (right at threshold) -> min frequency (1.0 Hz)
    pulse_gen.scale_urgency(0.20, 0.20, 1.0, 4.0);
    assert_eq!(pulse_gen.config.frequency, 1.0);

    // Health is 10% (halfway into danger) -> ~2.5 Hz
    pulse_gen.scale_urgency(0.10, 0.20, 1.0, 4.0);
    assert!((pulse_gen.config.frequency - 2.5).abs() < 0.01);

    // Health is 0% (critical death's door) -> max frequency (4.0 Hz)
    pulse_gen.scale_urgency(0.0, 0.20, 1.0, 4.0);
    assert_eq!(pulse_gen.config.frequency, 4.0);
}

#[test]
fn test_audio_trigger_edge_transitions() {
    let mut audio_mgr = AudioTriggerManager::new();
    let mut vitals = PlayerVitals::default();
    let mut vehicle = VehicleTelemetry::default();

    // Normal state -> no cues
    let cues = audio_mgr.update(Some(&vitals), Some(&vehicle), 0.016);
    assert!(cues.is_empty());

    // Transition health below 20%
    vitals.health.set(15.0);
    let cues = audio_mgr.update(Some(&vitals), Some(&vehicle), 0.016);
    assert!(cues.contains(&AudioCue::WarningBeep));

    // Next frame while still low health -> WarningBeep does NOT spam
    let cues = audio_mgr.update(Some(&vitals), Some(&vehicle), 0.016);
    assert!(!cues.contains(&AudioCue::WarningBeep));

    // Start moving fast without seatbelt -> seatbelt chime triggers after threshold
    vehicle.speed = Speed::from_mph(25.0);
    vehicle.seatbelt = false;
    let cues = audio_mgr.update(Some(&vitals), Some(&vehicle), 1.6);
    assert!(cues.contains(&AudioCue::SeatbeltChime));
}
