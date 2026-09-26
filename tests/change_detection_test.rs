use minimal_hud_rs::prelude::*;

#[test]
fn test_epsilon_dirty_checking() {
    let mut tracker = HudTracker::new(ChangeDetectionConfig {
        vitals_epsilon: 0.5,
        vitals_throttle_seconds: 0.0, // test epsilon directly without time gating
        ..Default::default()
    });

    let mut vitals = PlayerVitals::default();

    // First frame initializes and sets ALL_VITALS
    let flags = tracker.evaluate_vitals(&vitals, 0.016);
    assert_eq!(flags, DirtyFlags::ALL_VITALS);
    tracker.clear_dirty();

    // Microscopic jitter below epsilon (e.g. 0.2% change vs 0.5% threshold)
    vitals.health.set(99.8);
    let flags = tracker.evaluate_vitals(&vitals, 0.016);
    assert_eq!(flags, DirtyFlags::NONE);
    assert!(!tracker.is_dirty());

    // Significant change above epsilon (e.g. damage of 5%)
    vitals.health.set(94.8);
    let flags = tracker.evaluate_vitals(&vitals, 0.016);
    assert_eq!(flags, DirtyFlags::HEALTH);
    assert!(tracker.is_dirty());
}

#[test]
fn test_rate_limiting_and_throttling() {
    let mut tracker = HudTracker::new(ChangeDetectionConfig {
        vitals_epsilon: 0.1,
        vitals_throttle_seconds: 0.5, // 2 Hz updates max
        ..Default::default()
    });

    let mut vitals = PlayerVitals::default();
    tracker.evaluate_vitals(&vitals, 0.016);
    tracker.clear_dirty();

    // Rapid successive ticks within throttle interval
    vitals.hunger.set(80.0);
    for _ in 0..10 {
        let flags = tracker.evaluate_vitals(&vitals, 0.033);
        assert_eq!(flags, DirtyFlags::NONE, "Updates should be throttled");
    }

    // Now advance time past the throttle interval (0.5s)
    let flags = tracker.evaluate_vitals(&vitals, 0.2);
    assert_eq!(
        flags,
        DirtyFlags::HUNGER,
        "Should trigger dirty update once throttle window expires"
    );
}

#[test]
fn test_render_savings_ratio() {
    let mut tracker = HudTracker::new(ChangeDetectionConfig {
        vitals_epsilon: 0.25,
        vitals_throttle_seconds: 1.0, // 1 Hz vitals update
        ..Default::default()
    });

    let mut vitals = PlayerVitals::default();
    // Simulate 60 FPS for 10 seconds = 600 frames
    // Hunger decays very slowly
    for _i in 0..600 {
        vitals.hunger.modify(-0.01);
        tracker.evaluate_vitals(&vitals, 1.0 / 60.0);
        tracker.clear_dirty();
    }

    // Total evaluations = 600
    // Redraws should only occur roughly 10 times (once every 1 second)
    assert_eq!(tracker.total_evaluations(), 600);
    assert!(
        tracker.dispatched_renders() <= 12,
        "Dispatched: {}",
        tracker.dispatched_renders()
    );
    assert!(
        tracker.render_savings_percentage() >= 95.0,
        "Savings: {}%",
        tracker.render_savings_percentage()
    );
}

#[test]
fn test_exponential_smoothing_convergence() {
    let mut smoother = ExpSmoother::new(0.0, 10.0);
    let target = 100.0;
    let dt = 1.0 / 60.0;

    // After 1 frame, display value smoothly moves toward target
    smoother.update(target, dt);
    assert!(smoother.current() > 0.0 && smoother.current() < target);

    // After 0.5 seconds (~30 frames), it should be very close to target
    for _ in 0..30 {
        smoother.update(target, dt);
    }
    assert!((smoother.current() - target).abs() < 1.0);

    // After 1 second, it should have snapped cleanly to target
    for _ in 0..30 {
        smoother.update(target, dt);
    }
    assert_eq!(smoother.current(), target);
}
