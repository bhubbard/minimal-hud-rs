use minimal_hud_rs::prelude::*;

#[test]
fn test_speed_unit_conversions() {
    let speed_mph = Speed::from_mph(60.0);
    // 60 mph should be ~96.56 km/h
    assert!((speed_mph.as_kmh() - 96.5606).abs() < 0.05);
    assert!((speed_mph.as_mph() - 60.0).abs() < 0.01);
    assert_eq!(speed_mph.display_value(SpeedUnit::Mph), 60);
    assert_eq!(speed_mph.display_value(SpeedUnit::Kmh), 97);

    let speed_kmh = Speed::from_kmh(100.0);
    // 100 km/h should be ~62.137 mph
    assert!((speed_kmh.as_mph() - 62.137).abs() < 0.05);
    assert_eq!(speed_kmh.display_value(SpeedUnit::Kmh), 100);

    let speed_mps = Speed::from_mps(27.7778); // ~100 km/h
    assert!((speed_mps.as_kmh() - 100.0).abs() < 0.05);
}

#[test]
fn test_gear_representation() {
    assert_eq!(Gear::from_i8(-1).unwrap(), Gear::Reverse);
    assert_eq!(Gear::from_i8(0).unwrap(), Gear::Neutral);
    assert_eq!(Gear::from_i8(1).unwrap(), Gear::Forward(1));
    assert_eq!(Gear::from_i8(6).unwrap(), Gear::Forward(6));
    assert_eq!(Gear::from_i8(7).unwrap(), Gear::Forward(7));

    // Invalid gear checks
    assert!(Gear::from_i8(-2).is_err());
    assert!(Gear::from_i8(8).is_err());

    // Round-trip integer
    assert_eq!(Gear::Reverse.as_i8(), -1);
    assert_eq!(Gear::Neutral.as_i8(), 0);
    assert_eq!(Gear::Forward(4).as_i8(), 4);

    // Labels
    assert_eq!(Gear::Reverse.display_label(), "R");
    assert_eq!(Gear::Neutral.display_label(), "N");
    assert_eq!(Gear::Forward(3).display_label(), "3");
}

#[test]
fn test_engine_health_states() {
    let mut engine = EngineHealth::new(1000.0);
    assert_eq!(engine.state(), EngineState::Optimal);
    assert!(!engine.is_critical());

    engine.set(750.0);
    assert_eq!(engine.state(), EngineState::MinorDamage);

    engine.set(450.0);
    assert_eq!(engine.state(), EngineState::ModerateDamage);

    engine.set(250.0);
    assert_eq!(engine.state(), EngineState::Critical);
    assert!(engine.is_critical());

    engine.set(0.0);
    assert_eq!(engine.state(), EngineState::Destroyed);
    assert!(engine.is_critical());
}

#[test]
fn test_vehicle_alerts() {
    let mut telemetry = VehicleTelemetry::default();

    // Default stopped vehicle: handbrake is on, seatbelt unbuckled, but speed is 0
    assert!(
        telemetry
            .active_alerts()
            .contains(VehicleAlerts::HANDBRAKE_ENGAGED)
    );
    assert!(
        !telemetry
            .active_alerts()
            .contains(VehicleAlerts::UNBUCKLED_SEATBELT)
    );

    // Moving without seatbelt triggers warning
    telemetry.speed = Speed::from_mph(15.0);
    assert!(
        telemetry
            .active_alerts()
            .contains(VehicleAlerts::UNBUCKLED_SEATBELT)
    );

    // Buckling seatbelt clears warning
    telemetry.seatbelt = true;
    assert!(
        !telemetry
            .active_alerts()
            .contains(VehicleAlerts::UNBUCKLED_SEATBELT)
    );

    // Low fuel warning
    telemetry.set_fuel(12.0);
    assert!(telemetry.active_alerts().contains(VehicleAlerts::LOW_FUEL));

    // Engine critical alert
    telemetry.engine.set(200.0);
    assert!(
        telemetry
            .active_alerts()
            .contains(VehicleAlerts::ENGINE_CRITICAL)
    );
}

#[test]
fn test_rpm_redline_and_lights() {
    let mut telemetry = VehicleTelemetry::default();
    telemetry.set_rpm(0.80);
    assert!(!telemetry.is_redlining());

    telemetry.set_rpm(0.92);
    assert!(telemetry.is_redlining());

    // Lights cycling
    let mut light = LightState::Off;
    assert!(!light.is_on());
    light = light.cycle_next();
    assert_eq!(light, LightState::LowBeam);
    assert!(light.is_on());
    light = light.cycle_next();
    assert_eq!(light, LightState::HighBeam);
    light = light.cycle_next();
    assert_eq!(light, LightState::Off);
}
