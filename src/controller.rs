use chrono::Utc;

#[derive(Clone, Debug)]
pub struct PidConfig {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub out_min: f64,
    pub out_max: f64,
}

impl PidConfig {
    pub fn new(kp: f64, ki: f64, kd: f64, out_min: f64, out_max: f64) -> Self {
        Self { kp, ki, kd, out_min, out_max }
    }
}

pub struct ThermostatController {
    cfg: PidConfig,
    integral: f64,
    last_error: Option<f64>,
    last_time: Option<i64>,
}

impl ThermostatController {
    pub fn new(cfg: PidConfig) -> Self {
        Self { cfg, integral: 0.0, last_error: None, last_time: None }
    }

    pub fn update(&mut self, measured: f64, dt_seconds: f64) -> f64 {
        // For simplicity dt_seconds is used; in real system use timestamps
        let setpoint = 22.0;
        let error = setpoint - measured;
        self.integral += error * dt_seconds;
        let derivative = if let Some(le) = self.last_error {
            (error - le) / dt_seconds
        } else { 0.0 };
        self.last_error = Some(error);
        self.last_time = Some(Utc::now().timestamp_millis());
        let mut out = self.cfg.kp * error + self.cfg.ki * self.integral + self.cfg.kd * derivative;
        if out.is_nan() || out.is_infinite() { out = 0.0; }
        if out < self.cfg.out_min { out = self.cfg.out_min; }
        if out > self.cfg.out_max { out = self.cfg.out_max; }
        out
    }
}
