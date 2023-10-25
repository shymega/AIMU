extern crate linux_embedded_hal as hal;
use super::{Data, Error, BMI, IMU};
use bmi160;
use glam::Vec3;
use std::fmt::Display;

pub type BMI160I2C = bmi160::Bmi160<bmi160::interface::I2cInterface<hal::I2cdev>>;

impl<CommE: Display, CsE: Display> From<bmi160::Error<CommE, CsE>> for Error {
    fn from(_: bmi160::Error<CommE, CsE>) -> Self {
        Self::Driver
    }
}

impl BMI<BMI160I2C> {
    const SEC_PER_TICK: f32 = 39e-6; // [s/tick]
    const BITMASK_24: u32 = 0xffffff;

    fn dt(&self, t: u32) -> f32 {
        Self::SEC_PER_TICK * ((Self::BITMASK_24 & t.wrapping_sub(self.t)) as f32)
    }
}

impl IMU for BMI<BMI160I2C> {
    fn new(i2c_dev: &str, i2c_addr: u8) -> Result<Self, Error> {
        Ok(Self {
            drv: bmi160::Bmi160::new_with_i2c(
                hal::I2cdev::new(i2c_dev).map_err(|_| Error::Driver)?,
                match i2c_addr {
                    0x68 => bmi160::SlaveAddr::default(),
                    0x69 => bmi160::SlaveAddr::Alternative(true),
                    _ => panic!("Invalid address: {}", i2c_addr),
                },
            ),
            acc_res: 0.,
            gyr_res: 0.,
            t: 0,
        })
    }

    fn init(&mut self) -> Result<(), Error> {
        println!(
            "chip_id: 0x{:x}",
            self.drv.chip_id().map_err(|_| Error::Driver)?
        );
        // occasionally, first attempt doesn't take
        for _ in 0..2 {
            let _ = self
                .drv
                .set_accel_power_mode(bmi160::AccelerometerPowerMode::Normal);
            let _ = self
                .drv
                .set_gyro_power_mode(bmi160::GyroscopePowerMode::Normal);
        }
        self.acc_res = 4.0 / (u16::MAX as f32); // [g/bit] resolution
        self.gyr_res = 3000.0 / (u16::MAX as f32); // [deg/s/bit] resolution
        Ok(())
    }

    fn data(&mut self) -> Result<Data, Error> {
        let sensel = bmi160::SensorSelector::new().accel().gyro().time();
        let d = self.drv.data(sensel).map_err(|_| Error::Driver)?;
        let a = d.accel.unwrap();
        let g = d.gyro.unwrap();
        let t = d.time.unwrap();
        let dt: f32 = self.dt(t);
        self.t = t;
        Ok(Data {
            a: Vec3::new(a.x as f32, a.y as f32, a.z as f32) * self.acc_res,
            g: Vec3::new(g.x as f32, g.y as f32, g.z as f32) * self.gyr_res,
            t: dt,
        })
    }
}
