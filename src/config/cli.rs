use crate::config::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CLI {
    /// IMU model
    #[arg(short='m', long, value_enum, default_value_t = crate::imu::IMUs::BMI260)]
    model: crate::imu::IMUs,
    /// I2C device path
    #[arg(short='d', long, default_value_t = String::from("/dev/i2c-2"))]
    i2c_dev: String,
    /// I2C device address [e.g., 0x68 or 0x69]
    #[arg(short = 'a', long, default_value_t = 0x69)]
    i2c_addr: u8,
    /// trigger event code
    #[arg(short='e', long, value_enum, default_value_t = crate::device::trigger::EventCode::AbsZ)]
    trig_event: crate::device::trigger::EventCode,
    /// [deg] acute angle between rear of screen and plane of keyboard
    #[arg(short = 'r', long, value_name = "DEGREES", default_value_t = 45.)]
    screen: f32,
    // /// [-] flattened 3x3 transformation matrix for mapping device axes
    // #[arg(short = 'o', long, num_args = 9, default_values_t = vec![1,0,0,0,-1,0,0,0,-1])]
    // orient: Vec<i8>, //using vec until arrays are supported by clap
    /// [-] motion scale factor
    #[arg(short = 's', long, default_value_t = 50.0)]
    scale: f32,
    /// [Hz] update frequency
    #[arg(short = 'f', long, default_value_t = 40.0)]
    freq: f32,
}

impl ConfigAIMU {
    pub fn from_cli() -> Self {
        let args = CLI::parse();
        Self {
            imu: ConfigIMU {
                model: Some(args.model),
                i2c_dev: args.i2c_dev,
                i2c_addr: args.i2c_addr,
            },
            device: ConfigDevice {
                screen: args.screen,
                trigger: None, // orient: args.orient.try_into().map_err(|_| ConfigError::CLIMapping)?,
            },
            user: ConfigUser {
                scale: args.scale,
                freq: args.freq,
            },
        }
    }
}
