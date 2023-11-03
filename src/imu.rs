#![allow(clippy::new_ret_no_self)]
#![allow(clippy::upper_case_acronyms)]
#[cfg(feature = "bmi160")]
pub mod bmi160;
#[cfg(feature = "bmi260")]
pub mod bmi260;
use anyhow::Result;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to initialize")]
    Init,
    #[error("failed to communicate")]
    Driver,
    #[error("invalid device path")]
    Path,
    #[error("unsupported model")]
    Model,
    #[error("unknown IMU error")]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg(feature = "cli")]
#[derive(clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum IMUs {
    #[cfg(feature = "bmi160")]
    BMI160,
    #[cfg(feature = "bmi260")]
    BMI260,
}

impl IMUs {
    #[cfg(feature = "dynamic")]
    pub fn new(cfg: &Config) -> Result<Box<dyn IMU>, Error> {
        match cfg.model {
            #[cfg(feature = "bmi160")]
            Self::BMI160 => Ok(Box::new(bmi160::BMI160::new(
                cfg.i2c_dev.to_str().unwrap(),
                cfg.i2c_addr,
            )?)),
            #[cfg(feature = "bmi260")]
            Self::BMI260 => Ok(Box::new(bmi260::BMI260::new(
                cfg.i2c_dev.to_str().unwrap(),
                cfg.i2c_addr,
            )?)),
            _ => Err(Error::Model),
        }
    }

    #[cfg(all(feature = "bmi160", not(feature = "dynamic")))]
    pub fn new(cfg: &Config) -> bmi160::BMI160 {
        IMU::new(cfg.i2c_dev.to_str().unwrap(), cfg.i2c_addr)?
    }

    #[cfg(all(feature = "bmi260", not(feature = "dynamic")))]
    pub fn new(cfg: &Config) -> bmi260::BMI260 {
        IMU::new(cfg.i2c_dev.to_str().unwrap(), cfg.i2c_addr)?
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub model: IMUs,
    pub i2c_dev: std::path::PathBuf,
    pub i2c_addr: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: IMUs::BMI260,
            i2c_dev: std::path::PathBuf::from("/dev/i2c-2"),
            i2c_addr: 0x69,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Data {
    pub a: Vec3,
    pub g: Vec3,
    pub t: f32,
}

pub trait IMU {
    fn new(i2c_dev: &str, i2c_addr: u8) -> Result<Self, Error>
    where
        Self: Sized;
    fn init(&mut self) -> Result<(), Error>;
    fn data(&mut self) -> Result<Data, Error>;
}

#[derive(Debug, Default)]
pub struct BMI<T> {
    pub drv: T,
    pub acc_res: f32,
    pub gyr_res: f32,
    pub t: u32,
}
