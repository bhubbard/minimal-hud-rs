//! Benchmark comparing `minimal-hud-rs` (Rust) vs original FiveM Lua + NUI/CEF minimal-hud.

use minimal_hud_rs::prelude::*;
use std::time::Instant;

fn main() {
    println!("============================================================");
    println!("    minimal-hud-rs (Rust) vs FiveM Lua + NUI/CEF (Chromium) ");
    println!("============================================================");

    // 1. Full HUD Evaluation (Vitals + Vehicle Telemetry + Dirty Checking)
    println!("\n--- 1. Full HUD Frame Evaluation & Dirty Bitmask Filtering ---");
    {
        let mut tracker = HudTracker::new(ChangeDetectionConfig {
            vitals_epsilon: 0.2,
            speed_epsilon: 0.5,
            rpm_epsilon: 0.02,
            fuel_epsilon: 0.25,
            vitals_throttle_seconds: 0.1, // 10 Hz vitals
            vehicle_throttle_seconds: 0.033, // 30 Hz vehicle
        });

        let mut vitals = PlayerVitals::default();
        let mut vehicle = VehicleTelemetry::default();
        let iterations = 2_000_000;
        let dt = 0.016;
        let start = Instant::now();
        let mut dirty_render_count = 0;

        for i in 0..iterations {
            // Simulate realistic game physics jitter and gradual stat decay
            let speed_raw = 60.0 + (i as f32 * 0.05).sin() * 5.0;
            vehicle.speed = Speed::from_mph(speed_raw);
            vehicle.rpm = 0.6 + (i as f32 * 0.1).sin() * 0.3;
            vehicle.fuel = (100.0 - (i as f32 * 0.0001)).max(0.0);

            if i % 60 == 0 {
                vitals.hunger.set((100.0 - (i as f32 * 0.001)).max(0.0));
                vitals.thirst.set((100.0 - (i as f32 * 0.0015)).max(0.0));
            }

            let vit_flags = tracker.evaluate_vitals(&vitals, dt);
            let veh_flags = tracker.evaluate_vehicle(&vehicle, dt);

            if !vit_flags.is_empty() || !veh_flags.is_empty() {
                dirty_render_count += 1;
            }
            tracker.clear_dirty();
        }

        std::hint::black_box(dirty_render_count);
        let elapsed = start.elapsed();
        let ns_per_eval = elapsed.as_nanos() as f64 / iterations as f64;
        let evals_per_sec = iterations as f64 / elapsed.as_secs_f64();
        let savings_pct = (1.0 - (dirty_render_count as f64 / iterations as f64)) * 100.0;

        println!(
            "HUD Frame Steps: {} | Time: {:.2?} | Latency: {:.2} ns/step | {:>10.0} steps/s",
            iterations, elapsed, ns_per_eval, evals_per_sec
        );
        println!(
            "Redraw Savings: {:.1}% skipped via epsilon dirty-checking ({} actual renders vs {} frames)",
            savings_pct, dirty_render_count, iterations
        );
    }

    // 2. Framerate-Independent Exponential Smoothing
    println!("\n--- 2. Framerate-Independent Exponential Smoothing ---");
    {
        let mut smoother = ExpSmoother::new(0.0, 10.0); // lambda = 10.0
        let iterations = 10_000_000;
        let dt = 0.016;
        let start = Instant::now();
        let mut sum_smoothed = 0.0f32;

        for i in 0..iterations {
            let target = (i % 100) as f32;
            let val = smoother.update(target, dt);
            sum_smoothed += val;
        }

        std::hint::black_box(sum_smoothed);
        let elapsed = start.elapsed();
        let ns_per_smooth = elapsed.as_nanos() as f64 / iterations as f64;
        let smooths_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "Smooth Steps: {} | Time: {:.2?} | Latency: {:.2} ns/step | {:>10.0} steps/s",
            iterations, elapsed, ns_per_smooth, smooths_per_sec
        );
    }

    // 3. Procedural Pulse & Edge-Triggered Audio Cue Evaluation
    println!("\n--- 3. Procedural Alert Pulse & Audio Trigger Cues ---");
    {
        let mut pulse_gen = PulseGenerator::new(PulseConfig {
            frequency: 1.2,
            amplitude: 0.5,
            offset: 0.5,
            duty_cycle: 0.5,
            waveform: PulseWaveform::Heartbeat,
        });
        let mut audio_mgr = AudioTriggerManager::new();
        let mut vitals = PlayerVitals::default();
        let mut vehicle = VehicleTelemetry::default();

        let iterations = 5_000_000;
        let dt = 0.016;
        let start = Instant::now();
        let mut cues_fired = 0;

        for i in 0..iterations {
            let _alpha = pulse_gen.tick(dt);
            vitals.health.set(if (i % 1000) < 200 { 15.0 } else { 100.0 });
            vehicle.seatbelt = (i % 2000) > 1000;

            let cues = audio_mgr.update(Some(&vitals), Some(&vehicle), dt);
            cues_fired += cues.len();
        }

        std::hint::black_box(cues_fired);
        let elapsed = start.elapsed();
        let ns_per_pulse = elapsed.as_nanos() as f64 / iterations as f64;
        let pulses_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "Pulse & Audio Steps: {} | Time: {:.2?} | Latency: {:.2} ns/step | {:>10.0} steps/s | Cues: {}",
            iterations, elapsed, ns_per_pulse, pulses_per_sec, cues_fired
        );
    }

    // 4. Vehicle Telemetry & Multi-Unit Speed Transformation
    println!("\n--- 4. Vehicle Telemetry & Multi-Unit Speed Transformation ---");
    {
        let iterations = 10_000_000;
        let start = Instant::now();
        let mut sum_speeds = 0.0f32;

        for i in 0..iterations {
            let mps = (i % 100) as f32 * 0.5;
            let speed = Speed::from_mps(mps);
            let mph = speed.as_mph();
            let kmh = speed.as_kmh();
            sum_speeds += mph + kmh;
        }

        std::hint::black_box(sum_speeds);
        let elapsed = start.elapsed();
        let ns_per_calc = elapsed.as_nanos() as f64 / iterations as f64;
        let calcs_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!(
            "Speed Conversions: {} | Time: {:.2?} | Latency: {:.2} ns/calc | {:>10.0} calcs/s",
            iterations, elapsed, ns_per_calc, calcs_per_sec
        );
    }

    println!("\n============================================================");
    println!("                      Benchmark Complete                    ");
    println!("============================================================");
}
