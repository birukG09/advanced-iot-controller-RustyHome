use crate::devices::DeviceManager;
use chrono::Utc;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub struct TelemetryWriter {
    rows: Arc<Mutex<Vec<String>>>,
    out_path: String,
}

impl TelemetryWriter {
    pub fn new() -> Self {
        Self { rows: Arc::new(Mutex::new(Vec::new())), out_path: "telemetry.jsonl".to_string() }
    }

    pub fn capture(&mut self, step: i32, devices: &DeviceManager) {
        let snap = devices.state_snapshot();
        let record = json!({
            "ts": Utc::now().to_rfc3339(),
            "step": step,
            "numeric": snap.numeric,
            "boolean": snap.boolean,
        });
        let s = serde_json::to_string(&record).unwrap();
        self.rows.lock().unwrap().push(s);
    }

    pub async fn flush(&mut self) {
        let rows = self.rows.lock().unwrap().clone();
        let mut f = File::create(&self.out_path).unwrap();
        for r in rows.iter() {
            writeln!(f, "{}", r).unwrap();
        }
        println!("Telemetry written to {}", self.out_path);
    }
}
