IoT Home Controller Simulator (Rust)
===================================

This project is a Rust-based simulator of a home IoT control system.
It includes:
- simulated sensors (temperature, humidity, motion)
- actuators (lights, HVAC, door locks)
- an event-driven scheduler and rule engine
- a simple MQTT-like in-process broker
- PID controller for thermostat
- scenario runner and CLI-based telemetry output

Usage:
  cargo run --release

NOTE:
This is meant as a simulator and code example for learning advanced control concepts in IoT.
