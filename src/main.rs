//! IoT Home Controller Simulator
//! Entry point and scenario orchestrator.

mod devices;
mod network;
mod controller;
mod rules;
mod scheduler;
mod telemetry;
mod helpers;

use devices::{DeviceManager, SensorReading};
use network::Broker;
use controller::{ThermostatController, PidConfig};
use rules::RuleEngine;
use scheduler::Scheduler;
use telemetry::TelemetryWriter;
use chrono::Utc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("Starting IoT Home Controller Simulator — {}", Utc::now());
    let mut broker = Broker::new();
    let mut devices = DeviceManager::new();
    let pid_cfg = PidConfig::new(1.2, 0.01, 0.6, 0.0, 100.0);
    let mut thermostat = ThermostatController::new(pid_cfg);
    let mut rules = RuleEngine::default();
    let mut scheduler = Scheduler::new();
    let mut telemetry = TelemetryWriter::new();

    // Register a few devices
    devices.register_temperature_sensor("living_room_temp");
    devices.register_humidity_sensor("living_room_humidity");
    devices.register_motion_sensor("hall_motion");
    devices.register_light_actuator("living_room_light");
    devices.register_hvac_actuator("main_hvac");
    devices.register_lock_actuator("front_door_lock");

    // Simple automation rule: turn on living room light when motion and after sunset (simulated)
    rules.add_rule(rules::Rule::simple(
        "motion_light",
        |state| {
            let motion = state.get_bool("hall_motion");
            let hour = state.get_i32("clock_hour");
            (motion && (hour >= 18 || hour <= 6))
        },
        |state, mgr| {
            mgr.set_actuator("living_room_light", true);
            state.set_bool("living_room_light", true);
        },
    ));

    // Thermostat rule: maintain 22°C
    rules.add_rule(rules::Rule::pid_temperature_target(
        "thermostat_maintain_22",
        "living_room_temp",
        22.0,
    ));

    // Schedule periodic tasks
    scheduler.add_periodic(Duration::from_secs(1), Box::new(|ctx| {
        Box::pin(async move {
            // Periodic sensor simulation tick
            ctx.devices.tick_sensors().await;
        })
    }));

    // Telemetry: print summary every 5 seconds
    scheduler.add_periodic(Duration::from_secs(5), Box::new(|ctx| {
        Box::pin(async move {
            let _ = ctx.devices.state_snapshot(); // capture to keep things moving
        })
    }));

    // Main loop: run for N iterations or until user stops.
    let total_steps = 300; // ~5 minutes at 1s tick
    for step in 0..total_steps {
        // Advance clock and simulate environment
        devices.advance_clock();

        // Simulate sensors publishing to broker
        let readings: Vec<SensorReading> = devices.read_all();
        for r in readings.iter() {
            broker.publish(&r.topic, serde_json::to_string(&r).unwrap());
        }

        // Run rules against current state snapshot
        let state = devices.state_snapshot();
        rules.eval(&state, &mut devices).await;

        // Run PID thermostat controller
        if let Some(temp) = devices.get_temperature("living_room_temp") {
            let action = thermostat.update(temp, 1.0);
            devices.set_hvac_power("main_hvac", action);
        }

        // Run scheduled tasks
        scheduler.run_tick(&mut devices).await;

        // Broker processes messages (in-process subscriptions)
        broker.process(&mut devices);

        // Telemetry
        telemetry.capture(step as i32, &devices);

        // Sleep to simulate real time (can be reduced for faster runs)
        sleep(Duration::from_millis(1000)).await;
    }

    println!("Simulation finished. Writing telemetry...");
    telemetry.flush().await;
}
