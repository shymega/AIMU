use clap::Parser;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to map cli to config struct")]
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
pub struct ConfigIMU {
    // pub model: String,
    pub i2c_dev: String,
    pub i2c_addr: u8,
}

#[derive(Debug, Default)]
pub struct ConfigDevice {
    pub screen: f32,
    // pub orient: [i8; 9],
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
                // model: String::from("bmi260"),
                i2c_dev: String::from("/dev/i2c-2"),
                i2c_addr: 0x69,
            },
            device: ConfigDevice {
                /// [deg] acute angle between plane of keyboard and rear of screen
                screen: 45.,
                // /// orientation array [xx, xy, xz, yx, yy, yz, zx, zy, zz]
                // orient: [1, 0, 0, 0, 1, 0, 0, 0, 1],
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct CLI {
    // /// IMU model: bmi160 || bmi260
    // #[arg(short='m', long, default_value_t = String::from("bmi260"))]
    // model: String,
    /// I2C device path
    #[arg(short='d', long, default_value_t = String::from("/dev/i2c-2"))]
    i2c_dev: String,
    /// I2C device address: 0x68 || 0x69
    #[arg(short = 'a', long, default_value_t = 0x69)]
    i2c_addr: u8,
    /// [deg] acute angle between rear of screen and plane of keyboard
    #[arg(short = 'r', long, value_name = "DEGREES", default_value_t = 45.)]
    screen: f32,
    // /// [-] flattened 3x3 transformation matrix for mapping device axes
    // #[arg(short = 'o', long, num_args = 9, default_values_t = vec![1,0,0,0,1,0,0,0,1])]
    // orient: Vec<i8>, //using vec until arrays are supported by clap
    /// [-] motion scale factor
    #[arg(short = 's', long, default_value_t = 50.0)]
    scale: f32,
    /// [Hz] update frequency
    #[arg(short = 'f', long, default_value_t = 40.0)]
    freq: f32,
}

impl ConfigAIMU {
    pub fn from_cli() -> Result<Self, ConfigError> {
        let cli = CLI::parse();
        Ok(Self {
            imu: ConfigIMU {
                // model: cli.model,
                i2c_dev: cli.i2c_dev,
                i2c_addr: cli.i2c_addr,
            },
            device: ConfigDevice {
                screen: cli.screen,
                // orient: cli.orient.try_into().map_err(|_| ConfigError::CLIMapping)?,
            },
            user: ConfigUser {
                scale: cli.scale,
                freq: cli.freq,
            },
        })
    }
}
