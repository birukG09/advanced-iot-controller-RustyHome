use crate::devices::{DeviceManager, DeviceState};
use std::collections::HashMap;

// Lightweight rule abstraction.
pub struct Rule {
    pub name: String,
    pub condition: Box<dyn Fn(&DeviceState) -> bool + Send + Sync>,
    pub action: Box<dyn Fn(&mut DeviceState, &mut DeviceManager) + Send + Sync>,
}

impl Rule {
    pub fn simple<F, G>(name: &str, cond: F, act: G) -> Self
    where
        F: Fn(&DeviceState) -> bool + Send + Sync + 'static,
        G: Fn(&mut DeviceState, &mut DeviceManager) + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            condition: Box::new(cond),
            action: Box::new(act),
        }
    }

    pub fn pid_temperature_target(name: &str, temp_key: &str, target: f64) -> Self {
        let key = temp_key.to_string();
        Self {
            name: name.to_string(),
            condition: Box::new(move |_state: &DeviceState| { true }),
            action: Box::new(move |state: &mut DeviceState, mgr: &mut DeviceManager| {
                let cur = state.get_f64(&key).unwrap_or(0.0);
                // naive action: if temp < target -> increase hvac, else decrease
                if cur < target {
                    mgr.set_hvac_power("main_hvac", 80.0);
                } else {
                    mgr.set_hvac_power("main_hvac", 10.0);
                }
                state.set_f64("thermostat_target", target);
            }),
        }
    }
}

pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}

impl RuleEngine {
    pub fn add_rule(&mut self, r: Rule) {
        self.rules.push(r);
    }

    pub async fn eval(&mut self, state: &DeviceState, mgr: &mut DeviceManager) {
        // clone names to avoid borrow issues
        for r in self.rules.iter() {
            if (r.condition)(state) {
                // take snapshot of state, apply action
                let mut s = mgr.state_snapshot();
                (r.action)(&mut s, mgr);
            }
        }
    }
}
