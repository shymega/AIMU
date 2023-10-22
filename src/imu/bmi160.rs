extern crate linux_embedded_hal as hal;
use crate::imu::{Data, IMUError, TriAx, BMI, IMU};
use bmi160;
use std::fmt::Display;

pub type BMI160I2C = bmi160::Bmi160<bmi160::interface::I2cInterface<hal::I2cdev>>;

impl<CommE: Display, CsE: Display> From<bmi160::Error<CommE, CsE>> for IMUError {
    fn from(_: bmi160::Error<CommE, CsE>) -> Self {
        Self::Driver
    }
}

impl<T: From<i16>> From<bmi160::Sensor3DData> for TriAx<T> {
    fn from(d: bmi160::Sensor3DData) -> Self {
        Self {
            x: <T>::from(d.x),
            y: <T>::from(d.y),
            z: <T>::from(d.z),
        }
    }
}

impl<T: From<i16>, U: From<u32>> From<bmi160::Data> for Data<T, U> {
    fn from(d: bmi160::Data) -> Self {
        Self {
            a: TriAx::<T>::from(d.accel.unwrap()),
            g: TriAx::<T>::from(d.gyro.unwrap()),
            t: <U>::from(d.time.unwrap()),
        }
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
    fn new(i2c_dev: &str, i2c_addr: u8) -> Result<Self, IMUError> {
        Ok(Self {
            drv: bmi160::Bmi160::new_with_i2c(
                hal::I2cdev::new(i2c_dev).map_err(|_| IMUError::Driver)?,
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

    fn init(&mut self) -> Result<(), IMUError> {
        println!(
            "chip_id: 0x{:x}",
            self.drv.chip_id().map_err(|_| IMUError::Driver)?
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

    fn data(&mut self) -> Result<Data<f32, f32>, IMUError> {
        let sensel = bmi160::SensorSelector::new().accel().gyro().time();
        let data = Data::from(self.drv.data(sensel).map_err(|_| IMUError::Driver)?);
        let dt: f32 = self.dt(data.t);
        self.t = data.t;
        Ok(Data {
            a: &data.a * self.acc_res,
            g: &data.g * self.gyr_res,
            t: dt,
        })
    }
}
