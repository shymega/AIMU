#![allow(clippy::new_ret_no_self)]
#![allow(clippy::upper_case_acronyms)]
#[cfg(any(feature = "bmi160", feature = "dynamic"))]
pub mod bmi160;
#[cfg(any(feature = "bmi260", feature = "dynamic"))]
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
    BMI160,
    BMI260,
}

impl IMUs {
    #[cfg(feature = "dynamic")]
    pub fn new(cfg: &Config) -> Result<Box<dyn IMU>, Error> {
        match cfg.model {
            Self::BMI160 => Ok(Box::new(BMI::<bmi160::BMI160I2C>::new(
                cfg.i2c_dev.to_str().unwrap(),
                cfg.i2c_addr,
            )?)),
            Self::BMI260 => Ok(Box::new(BMI::<bmi260::BMI260I2C>::new(
                cfg.i2c_dev.to_str().unwrap(),
                cfg.i2c_addr,
            )?)),
            _ => Err(Error::Model),
        }
    }

    #[cfg(all(feature = "bmi160", not(feature = "dynamic")))]
    pub fn new(cfg: &Config) -> BMI<bmi160::BMI160I2C> {
        IMU::new(cfg.i2c_dev.to_str().unwrap(), cfg.i2c_addr)?
    }

    #[cfg(all(feature = "bmi260", not(feature = "dynamic")))]
    pub fn new(cfg: &Config) -> BMI<bmi260::BMI260I2C> {
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
