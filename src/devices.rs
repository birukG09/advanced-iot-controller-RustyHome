use serde::{Deserialize, Serialize};
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub topic: String,
    pub value: f64,
    pub sensor_type: String,
}

#[derive(Default, Debug, Clone)]
pub struct DeviceState {
    pub numeric: HashMap<String, f64>,
    pub boolean: HashMap<String, bool>,
}

impl DeviceState {
    pub fn get_f64(&self, k: &str) -> Option<f64> {
        self.numeric.get(k).copied()
    }
    pub fn set_f64(&mut self, k: &str, v: f64) {
        self.numeric.insert(k.to_string(), v);
    }
    pub fn get_bool(&self, k: &str) -> bool {
        *self.boolean.get(k).unwrap_or(&false)
    }
    pub fn set_bool(&mut self, k: &str, v: bool) {
        self.boolean.insert(k.to_string(), v);
    }
    pub fn get_i32(&self, k: &str) -> i32 {
        self.numeric.get(k).map(|v| *v as i32).unwrap_or(0)
    }
}

pub struct DeviceManager {
    pub state: Arc<Mutex<DeviceState>>,
    // internal simulation knobs
    clock_hour: i32,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self { state: Arc::new(Mutex::new(DeviceState::default())), clock_hour: 12 }
    }

    pub fn register_temperature_sensor(&mut self, name: &str) {
        let mut s = self.state.lock().unwrap();
        s.set_f64(name, 20.0 + rand::thread_rng().gen_range(-1.0..1.0));
    }
    pub fn register_humidity_sensor(&mut self, name: &str) {
        let mut s = self.state.lock().unwrap();
        s.set_f64(name, 40.0 + rand::thread_rng().gen_range(-5.0..5.0));
    }
    pub fn register_motion_sensor(&mut self, name: &str) {
        let mut s = self.state.lock().unwrap();
        s.set_bool(name, false);
    }
    pub fn register_light_actuator(&mut self, name: &str) {
        let mut s = self.state.lock().unwrap();
        s.set_bool(name, false);
    }
    pub fn register_hvac_actuator(&mut self, name: &str) {
        let mut s = self.state.lock().unwrap();
        s.set_f64(name, 0.0); // power level 0..100
    }
    pub fn register_lock_actuator(&mut self, name: &str) {
        let mut s = self.state.lock().unwrap();
        s.set_bool(name, true); // locked
    }

    pub fn read_all(&self) -> Vec<SensorReading> {
        let s = self.state.lock().unwrap();
        let mut out = Vec::new();
        for (k, v) in s.numeric.iter() {
            out.push(SensorReading { topic: format!("sensors/{}", k), value: *v, sensor_type: "numeric".to_string() });
        }
        for (k, v) in s.boolean.iter() {
            out.push(SensorReading { topic: format!("sensors/{}", k), value: if *v { 1.0 } else { 0.0 }, sensor_type: "boolean".to_string() });
        }
        out
    }

    pub fn advance_clock(&mut self) {
        self.clock_hour = (self.clock_hour + 1) % 24;
        let mut s = self.state.lock().unwrap();
        s.set_f64("clock_hour", self.clock_hour as f64);
        // Simulate temperature drift
        let temp_key = "living_room_temp".to_string();
        if let Some(cur) = s.get_f64(&temp_key) {
            let noise: f64 = rand::thread_rng().gen_range(-0.2..0.2);
            // If HVAC power is on, influence temperature
            let hvac_power = s.get_f64("main_hvac").unwrap_or(0.0);
            let ambient = 18.0 + ((self.clock_hour as f64 - 12.0) / 12.0) * 4.0;
            let new_temp = cur + (ambient - cur) * 0.02 + (hvac_power - 50.0) * 0.005 + noise;
            s.set_f64(&temp_key, new_temp);
        }
        // Random motion events at night
        if self.clock_hour >= 20 || self.clock_hour <= 6 {
            if rand::thread_rng().gen_bool(0.05) {
                s.set_bool("hall_motion", true);
            } else {
                s.set_bool("hall_motion", false);
            }
        } else {
            s.set_bool("hall_motion", false);
        }
    }

    pub fn state_snapshot(&self) -> DeviceState {
        self.state.lock().unwrap().clone()
    }

    pub fn get_temperature(&self, key: &str) -> Option<f64> {
        self.state.lock().unwrap().get_f64(key)
    }

    pub fn set_hvac_power(&mut self, key: &str, power: f64) {
        let mut s = self.state.lock().unwrap();
        s.set_f64(key, power.max(0.0).min(100.0));
    }

    pub fn set_actuator(&mut self, key: &str, on: bool) {
        let mut s = self.state.lock().unwrap();
        s.set_bool(key, on);
    }

    // Helpers used by scheduler/context
    pub async fn tick_sensors(&mut self) {
        // small read and possible state changes
        let mut s = self.state.lock().unwrap();
        // humidity drift slightly
        if let Some(h) = s.get_f64("living_room_humidity") {
            let newh = (h + rand::thread_rng().gen_range(-0.5..0.5)).clamp(20.0, 80.0);
            s.set_f64("living_room_humidity", newh);
        }
    }
}
