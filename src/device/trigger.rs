use evdev::{self, AbsoluteAxisType, Device, InputEventKind};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[cfg(feature = "cli")]
#[derive(clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum EventCode {
    AbsZ,
}

//TODO: support code combos, e.g., ABS_RX|ABS_RY
impl From<EventCode> for InputEventKind {
    fn from(val: EventCode) -> Self {
        match val {
            EventCode::AbsZ => InputEventKind::AbsAxis(AbsoluteAxisType::ABS_Z),
            _ => panic!("unsupported event code!"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub device: Option<String>,
    pub event: EventCode,
    pub thresh: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device: Some(String::from("Microsoft X-Box 360 pad")),
            event: EventCode::AbsZ,
            thresh: 10,
        }
    }
}

pub struct Trigger {
    device: Option<Arc<Mutex<Device>>>,
    event: Arc<Mutex<InputEventKind>>,
    thresh: i32,
    state: Arc<Mutex<i32>>,
}

// impl From<Config> for Trigger {
//     fn from(val: Config) -> Self {
//         Self::new(&val.device, val.event, val.thresh)
//     }
// }

impl Trigger {
    pub fn new(cfg: Config) -> Self {
        Self {
            device: cfg.device.and_then(|dev| {
                evdev::enumerate()
                    .map(|t| t.1)
                    .find(|d| d.name().unwrap() == dev)
                    .and_then(|dev| Some(Arc::new(Mutex::new(dev))))
            }),
            event: Arc::new(Mutex::new(cfg.event.into())),
            thresh: cfg.thresh,
            state: Arc::new(Mutex::new(0)),
        }
    }

    pub fn task(&self) {
        let state = Arc::clone(&self.state);
        match &self.device {
            Some(dev) => {
                let device = Arc::clone(&(dev));
                let event = Arc::clone(&self.event);
                thread::spawn(move || loop {
                    for e in device.lock().unwrap().fetch_events().unwrap() {
                        if e.kind() == *event.lock().unwrap() {
                            *(state.lock().unwrap()) = e.value();
                        };
                    }
                });
            }
            None => *(state.lock().unwrap()) = self.thresh,
        };
    }

    pub fn check(&self) -> bool {
        *self.state.lock().unwrap() >= self.thresh
    }
}
