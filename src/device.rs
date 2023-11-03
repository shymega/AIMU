use anyhow::Result;
use serde::{Deserialize, Serialize};
pub mod trigger;
pub mod vmouse;

pub trait VDev {
    fn new() -> Result<Self>
    where
        Self: Sized;
    fn update(&mut self, x: i32, y: i32) -> Result<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub trigger: trigger::Config,
    // pub vdev: dyn VDev,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            trigger: trigger::Config::default(),
        }
    }
}
