#![allow(clippy::upper_case_acronyms)]
use crate::config::*;
use crate::device::trigger;
use crate::imu;
use clap::{Args, Parser};

#[derive(Args, Debug)]
#[group(required = false, requires_all = ["model", "i2c_dev", "i2c_addr"])]
struct IMU {
    /// IMU model
    #[arg(long = "imu_model", value_enum, default_value_t = imu::IMUs::BMI260)]
    model: imu::IMUs,
    /// IMU I2C device path
    #[arg(long, default_value_os_t = imu::Config::default().i2c_dev)]
    i2c_dev: std::path::PathBuf,
    /// IMU I2C device address [e.g.: 0x68 (104) or 0x69 (105)]
    #[arg(long, default_value_t = imu::Config::default().i2c_addr)]
    i2c_addr: u8,
}

// impl std::fmt::Display for Option<String> {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{}", self.unwrap_or("None"))
//     }
// }

#[derive(Args, Debug, Clone)]
#[group(required = false)]
struct Trigger {
    /// trigger device name
    // #[arg(long = "trigger", default_value_t = trigger::Config::default().device.unwrap())]
    // device: String,
    /// trigger device name [e.g.: "Microsoft X-Box 360 pad"]
    #[arg(long = "trigger")]
    device: Option<String>,
    /// trigger event code
    #[arg(long = "event", value_enum, default_value_t = trigger::Config::default().event)]
    event: trigger::EventCode,
    /// trigger state transition threshold
    #[arg(long = "thresh", default_value_t = trigger::Config::default().thresh)]
    thresh: i32,
}

impl From<Trigger> for trigger::Config {
    fn from(val: Trigger) -> Self {
        Self {
            // device: Some(val.device),
            device: val.device,
            event: val.event,
            thresh: val.thresh,
        }
    }
}

impl From<Trigger> for trigger::Trigger {
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
    #[arg(short = 'd', long, value_name = "DEGREES", default_value_t = 45.)]
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
