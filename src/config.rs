#[cfg(feature = "cli")]
pub mod cli;

use crate::device::Config as ConfigDevice;
use crate::imu::Config as ConfigIMU;
use crate::motion::Config as ConfigMotion;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to map cli args to config struct")]
    Mapping,
    #[error("unknown config error")]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub imu: ConfigIMU,
    pub device: ConfigDevice,
    pub motion: ConfigMotion,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            imu: ConfigIMU {
                model: crate::imu::Unit::BMI260,
                i2c_dev: PathBuf::from("/dev/i2c-2"),
                i2c_addr: 0x69,
            },
            device: ConfigDevice::default(),
            motion: ConfigMotion::default(),
        }
    }
}
