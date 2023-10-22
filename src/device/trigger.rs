use evdev::{self, AbsoluteAxisType, Device, InputEventKind};
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone, Debug)]
#[cfg(feature = "cli")]
#[derive(clap::ValueEnum)]
pub enum EventCode {
    AbsZ,
    None,
}

#[derive(Debug)]
pub struct Config {
    pub thresh: u8,
    pub device: String,
    pub code: EventCode,
    // code: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            thresh: 10,
            device: String::from("Microsoft X-Box 360 pad"),
            code: EventCode::AbsZ,
            // code: vec![String::from("ABS_RX"), String::from("ABS_RY")],
        }
    }
}

//TODO: support code combos, e.g., ABS_RX|ABS_RY
impl Config {
    fn parse_code(&self) -> InputEventKind {
        match self.code {
            EventCode::AbsZ => InputEventKind::AbsAxis(AbsoluteAxisType::ABS_Z),
            _ => panic!("invalid code"),
        }
    }
}

pub struct Trigger {
    device: Arc<Mutex<Device>>,
    event: Arc<Mutex<InputEventKind>>,
    thresh: i32,
    state: Arc<Mutex<i32>>,
}

impl Trigger {
    pub fn new(device: &str, event: InputEventKind, thresh: i32) -> Self {
        Self {
            device: Arc::new(Mutex::new(
                evdev::enumerate()
                    .map(|t| t.1)
                    .find(|d| d.name().unwrap() == device)
                    .unwrap(),
            )),
            event: Arc::new(Mutex::new(event)),
            thresh,
            state: Arc::new(Mutex::new(0)),
        }
    }

    pub fn task(&self) {
        let state = Arc::clone(&self.state);
        let device = Arc::clone(&self.device);
        let event = Arc::clone(&self.event);
        thread::spawn(move || loop {
            for e in device.lock().unwrap().fetch_events().unwrap() {
                if e.kind() == *event.lock().unwrap() {
                    *(state.lock().unwrap()) = e.value();
                };
            }
        });
    }

    pub fn check(&self) -> bool {
        *self.state.lock().unwrap() > self.thresh
    }
}
