use crate::devices::DeviceManager;
use std::collections::HashMap;

pub struct Broker {
    // Simple in-process pub/sub: topic -> Vec<callback identifiers>
    subscribers: HashMap<String, Vec<String>>,
    messages: Vec<(String, String)>, // (topic, payload)
}

impl Broker {
    pub fn new() -> Self {
        Self { subscribers: HashMap::new(), messages: Vec::new() }
    }

    pub fn subscribe(&mut self, topic: &str, id: &str) {
        let v = self.subscribers.entry(topic.to_string()).or_default();
        if !v.contains(&id.to_string()) {
            v.push(id.to_string());
        }
    }

    pub fn publish(&mut self, topic: &str, payload: String) {
        self.messages.push((topic.to_string(), payload));
    }

    pub fn process(&mut self, devices: &mut DeviceManager) {
        // process queued messages and apply simple handlers to device manager
        while let Some((topic, payload)) = self.messages.pop() {
            // for simulation, we inspect topic prefixes
            if topic.starts_with("sensors/") {
                // ignore ... sensor messages already represented in state
            } else if topic.starts_with("commands/") {
                // very simple: commands/<actuator> -> "on"/"off" or json with power
                let parts: Vec<&str> = topic.split('/').collect();
                if parts.len() >= 2 {
                    let act = parts[1];
                    if payload == "on" {
                        devices.set_actuator(act, true);
                    } else if payload == "off" {
                        devices.set_actuator(act, false);
                    } else if let Ok(v) = payload.parse::<f64>() {
                        devices.set_hvac_power(act, v);
                    }
                }
            }
        }
    }
}
