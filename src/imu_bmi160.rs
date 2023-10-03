extern crate linux_embedded_hal as hal;
use std::{ops::Mul, thread::sleep, time::Duration};

use crate::imu;
use bmi160;
use imu::{Data, TriAx, IMU};
pub type BMI160I2C = bmi160::Bmi160<bmi160::interface::I2cInterface<hal::I2cdev>>;

impl<T> From<bmi160::Sensor3DData> for TriAx<T>
where
    T: From<i16>,
{
    fn from(d: bmi160::Sensor3DData) -> Self {
        Self {
            x: <T>::from(d.x),
            y: <T>::from(d.y),
            z: <T>::from(d.z),
        }
    }
}

impl<T, U> From<bmi160::Data> for Data<T, U>
where
    T: From<i16>,
    U: From<u32>,
{
    fn from(d: bmi160::Data) -> Self {
        Self {
            a: TriAx::<T>::from(d.accel.unwrap()),
            g: TriAx::<T>::from(d.gyro.unwrap()),
            t: <U>::from(d.time.unwrap()),
        }
    }
}

pub struct BMI160 {
    drv: BMI160I2C,
    acc_res: f32,
    gyr_res: f32,
    t: u32,
}

impl BMI160 {
    const SEC_PER_TICK: f32 = 39e-6; // [s/tick]
    const BITMASK_24: u32 = 0xffffff;
}

impl IMU for BMI160 {
    fn new(i2c_dev: String, i2c_addr: u8) -> Self {
        BMI160 {
            drv: bmi160::Bmi160::new_with_i2c(
                hal::I2cdev::new(i2c_dev.as_str()).unwrap(),
                match i2c_addr {
                    0x68 => bmi160::SlaveAddr::default(),
                    0x69 => bmi160::SlaveAddr::Alternative(true),
                    _ => panic!("Invalid address: {}", i2c_addr),
                },
            ),
            t: 0,
            acc_res: 0.,
            gyr_res: 0.,
        }
    }

    fn init(&mut self) {
        println!("chip_id: 0x{:x}", self.drv.chip_id().unwrap());

        // occasionally, first attempt doesn't take
        for _ in 0..2 {
            self.drv
                .set_accel_power_mode(bmi160::AccelerometerPowerMode::Normal);
            self.drv
                .set_gyro_power_mode(bmi160::GyroscopePowerMode::Normal);
        }

        self.acc_res = 4.0 / (u16::MAX as f32); // [g/bit] resolution
        self.gyr_res = 3000.0 / (u16::MAX as f32); // [deg/s/bit] resolution
    }

    fn data(&mut self) -> Data<f32, f32> {
        let sensel = bmi160::SensorSelector::new().accel().gyro().time();
        let mut data = Data::from(self.drv.data(sensel).unwrap());
        let dt: f32 = self.dt(data.t);
        self.t = data.t;
        Data {
            a: &data.a * self.acc_res,
            g: &data.g * self.gyr_res,
            t: dt,
        }
    }

    fn dt(&self, t: u32) -> f32 {
        Self::SEC_PER_TICK * ((Self::BITMASK_24 & t.wrapping_sub(self.t)) as f32)
    }
}
