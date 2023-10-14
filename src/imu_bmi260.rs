extern crate linux_embedded_hal as hal;
use crate::imu;
use imu::{Data, IMUError, TriAx, BMI, IMU};
use std::{ops::Mul, thread::sleep, time::Duration};
use thiserror::Error;

use bmi270;
pub type BMI260I2C = bmi270::Bmi270<bmi270::interface::I2cInterface<hal::I2cdev>>;

impl<CommE, CsE> From<bmi270::Error<CommE, CsE>> for IMUError {
    fn from(_: bmi270::Error<CommE, CsE>) -> Self {
        Self::Driver
    }
}

impl<T: From<i16>> From<bmi270::AxisData> for TriAx<T> {
    fn from(d: bmi270::AxisData) -> Self {
        Self {
            x: <T>::from(d.x),
            y: <T>::from(d.y),
            z: <T>::from(d.z),
        }
    }
}

impl<T: From<i16>, U: From<u32>> From<bmi270::Data> for Data<T, U> {
    fn from(d: bmi270::Data) -> Self {
        Self {
            a: TriAx::<T>::from(d.acc),
            g: TriAx::<T>::from(d.gyr),
            t: <U>::from(d.time),
        }
    }
}

impl BMI<BMI260I2C> {
    const SEC_PER_TICK: f32 = 39e-6; // [s/tick]

    fn reset(&mut self) -> Result<(), IMUError> {
        self.drv.send_cmd(bmi270::Cmd::SoftReset)?;
        sleep(Duration::from_millis(10));
        Ok(())
    }

    fn acc_range(&mut self) -> Result<u8, IMUError> {
        // [g] +/- range (i.e., half of span)
        Ok(1 << (1 + self.drv.get_acc_range()? as u8))
    }

    fn gyr_range(&mut self) -> Result<u16, IMUError> {
        // [deg/s] +/- range (i.e., half of span)
        Ok(2000 >> (self.drv.get_gyr_range()?.range as u8))
    }
}

impl IMU<IMUError> for BMI<BMI260I2C> {
    fn new(i2c_dev: &str, i2c_addr: u8) -> Self {
        Self {
            drv: bmi270::Bmi270::new_i2c(
                hal::I2cdev::new(i2c_dev).unwrap(),
                match i2c_addr {
                    0x68 => bmi270::I2cAddr::Default,
                    0x69 => bmi270::I2cAddr::Alternative,
                    _ => panic!("Invalid address: {}", i2c_addr),
                },
                bmi270::Burst::Other(255),
            ),
            acc_res: 0.,
            gyr_res: 0.,
            t: 0,
        }
    }

    fn init(&mut self) -> Result<(), IMUError> {
        println!("chip_id: 0x{:x}", self.drv.get_chip_id()?);
        self.reset()?;
        self.drv.init(&bmi270::config::BMI260_CONFIG_FILE)?;
        let acc_range = self.acc_range()?;
        let gyr_range = self.gyr_range()?;
        println!("acc_range: ±{} g", acc_range);
        println!("gyr_range:  ±{} °/s", gyr_range);
        self.acc_res = ((acc_range << 1) as f32) / (u16::MAX as f32); // [g/bit] resolution
        self.gyr_res = ((gyr_range << 1) as f32) / (u16::MAX as f32); // [deg/s/bit] resolution

        let pwr_ctrl = bmi270::PwrCtrl {
            aux_en: false,
            gyr_en: true,
            acc_en: true,
            temp_en: false,
        };
        self.drv.set_pwr_ctrl(pwr_ctrl)?;
        Ok(())
    }

    fn data(&mut self) -> Result<Data<f32, f32>, IMUError> {
        let mut data = Data::from(self.drv.get_data()?);
        let dt: f32 = self.dt(data.t);
        self.t = data.t;
        Ok(Data {
            a: &data.a * self.acc_res,
            g: &data.g * self.gyr_res,
            t: dt,
        })
    }

    fn dt(&self, t: u32) -> f32 {
        Self::SEC_PER_TICK * (t.wrapping_sub(self.t) as f32)
    }
}
