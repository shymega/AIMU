#![allow(clippy::upper_case_acronyms)]
use crate::config::*;
use clap::{Args, Parser};

#[derive(Args, Debug)]
#[group(required = false, requires_all = ["model", "i2c_dev", "i2c_addr"])]
struct IMU {
    /// IMU model
    #[arg(short='m', long, value_enum, default_value_t = crate::imu::IMUs::BMI260)]
    model: crate::imu::IMUs,
    /// IMU I2C device path
    #[arg(short='d', long, default_value_os_t = crate::imu::Config::default().i2c_dev)]
    i2c_dev: std::path::PathBuf,
    /// IMU I2C device address [e.g., 0x68 (104) or 0x69 (105)]
    #[arg(short = 'a', long, default_value_t = crate::imu::Config::default().i2c_addr)]
    i2c_addr: u8,
}

#[derive(Args, Debug, Clone)]
#[group(required = false)]
struct Trigger {
    /// trigger device name
    #[arg(short = 't', long="trig_dev", default_value_t = crate::device::trigger::Config::default().device)]
    device: String,
    /// trigger event code
    #[arg(short = 'e', long = "trig_ev", value_enum, default_value_t = crate::device::trigger::Config::default().event)]
    event: crate::device::trigger::EventCode,
    /// trigger state transition threshold
    #[arg(short = 'r', long = "trig_thresh", default_value_t = crate::device::trigger::Config::default().thresh)]
    thresh: i32,
}

impl From<Trigger> for crate::device::trigger::Config {
    fn from(val: Trigger) -> Self {
        Self {
            device: val.device,
            event: val.event,
            thresh: val.thresh,
        }
    }
}
impl From<Trigger> for crate::device::trigger::Trigger {
    fn from(val: Trigger) -> Self {
        Self::new(val.into())
    }
}

#[derive(Args, Debug)]
#[group(required = false)]
struct Device {
    #[command(flatten)]
    trigger: Trigger,
    /// [deg] acute angle between rear of screen and plane of keyboard
    #[arg(short = 'g', long, value_name = "DEGREES", default_value_t = 45.)]
    screen: f32,
    // /// [-] flattened 3x3 transformation matrix for mapping device axes
    // #[arg(short = 'o', long, num_args = 9, default_values_t = vec![1,0,0,0,-1,0,0,0,-1])]
    // orient: Vec<i8>, //using vec until arrays are supported by clap
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CLI {
    #[command(flatten)]
    imu: IMU,
    #[command(flatten)]
    device: Device,
    /// [-] motion scale factor
    #[arg(short = 's', long, default_value_t = 50.0)]
    scale: f32,
    /// [Hz] update frequency
    #[arg(short = 'f', long, default_value_t = 40.0)]
    freq: f32,
}

impl Config {
    pub fn from_cli() -> Self {
        let args = CLI::parse();
        Self {
            imu: ConfigIMU {
                model: args.imu.model,
                i2c_dev: args.imu.i2c_dev,
                i2c_addr: args.imu.i2c_addr,
            },
            device: ConfigDevice {
                screen: args.device.screen,
                trigger: args.device.trigger.into(),
                // orient: args.orient.try_into().map_err(|_| ConfigError::CLIMapping)?,
            },
            user: ConfigUser {
                scale: args.scale,
                freq: args.freq,
            },
        }
    }
}
