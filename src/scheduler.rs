use crate::devices::DeviceManager;
use std::time::Duration;
use futures::Future;
use std::pin::Pin;

pub type TaskFn = Box<dyn Fn(&mut SchedulerContext) -> Pin<Box<dyn futures::Future<Output = ()> + Send>> + Send + Sync>;

pub struct Scheduled {
    pub interval: Duration,
    pub func: TaskFn,
    pub last_run: Option<std::time::Instant>,
}

pub struct Scheduler {
    tasks: Vec<Scheduled>,
}

pub struct SchedulerContext<'a> {
    pub devices: &'a mut DeviceManager,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_periodic(&mut self, interval: Duration, func: TaskFn) {
        self.tasks.push(Scheduled { interval, func, last_run: None });
    }

    pub async fn run_tick(&mut self, devices: &mut DeviceManager) {
        let now = std::time::Instant::now();
        for t in self.tasks.iter_mut() {
            let do_run = match t.last_run {
                None => true,
                Some(lr) => now.duration_since(lr) >= t.interval,
            };
            if do_run {
                t.last_run = Some(now);
                let mut ctx = SchedulerContext { devices };
                (t.func)(&mut ctx).await;
            }
        }
    }
}
