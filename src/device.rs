use anyhow::Result;
use glam::Mat3;
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
    /// [deg] acute angle between plane of keyboard and rear of screen
    pub screen: f32,
    /// orientation array [xx, xy, xz, yx, yy, yz, zx, zy, zz]
    pub orient: Mat3,
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
