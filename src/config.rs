#[cfg(feature = "cli")]
pub mod cli;

use crate::device::Config as ConfigDevice;
use crate::imu::Config as ConfigIMU;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to map cli args to config struct")]
    CLIMapping,
    #[error("unknown config error")]
    Unknown,
}

#[derive(Debug)]
pub struct ConfigAIMU {
    pub imu: ConfigIMU,
    pub device: ConfigDevice,
    pub user: ConfigUser,
}

#[derive(Debug, Default)]
pub struct ConfigUser {
    pub scale: f32,
    pub freq: f32,
    // pub frame: String,
}

impl Default for ConfigAIMU {
    fn default() -> Self {
        Self {
            imu: ConfigIMU {
                model: Some(crate::imu::IMUs::BMI260),
                i2c_dev: PathBuf::from("/dev/i2c-2")
                    .into_os_string()
                    .into_string()
                    .unwrap(),
                i2c_addr: 0x69,
            },
            device: ConfigDevice {
                /// [deg] acute angle between plane of keyboard and rear of screen
                screen: 45.,
                // /// orientation array [xx, xy, xz, yx, yy, yz, zx, zy, zz]
                // orient: [1, 0, 0, 0, -1, 0, 0, 0, -1],
                trigger: None,
            },
            user: ConfigUser {
                /// [-] arbitrary scale factor
                scale: 50.0,
                /// [Hz] update frequency
                freq: 40.0,
                // frame of reference for processing motion control
                // frame: String::from("local"),
            },
        }
    }
}
