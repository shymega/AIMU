extern crate linux_embedded_hal as hal;
use bmi160;
use bmi270;
use hal::i2cdev::core::I2CDevice;
use std::{error::Error, thread::sleep, time::Duration};

pub struct IMU<T> {
    imu: T,
    acc_res: f32,
    gyr_res: f32,
}

pub type BMI160 = bmi160::Bmi160<bmi160::interface::I2cInterface<hal::I2cdev>>;
pub type BMI260 = bmi270::Bmi270<bmi270::interface::I2cInterface<hal::I2cdev>>;

impl IMU<BMI260> {
    const SEC_PER_TICK: f32 = 39e-6; // [s/tick]

    pub fn new(i2c_dev: String, i2c_addr: u8) -> Self {
        IMU {
            imu: bmi270::Bmi270::new_i2c(
                hal::I2cdev::new(i2c_dev.as_str()).unwrap(),
                match i2c_addr {
                    0x68 => bmi270::I2cAddr::Default,
                    0x69 => bmi270::I2cAddr::Alternative,
                    _ => panic!("Invalid address: {}", i2c_addr),
                },
                bmi270::Burst::Other(255),
            ),
            acc_res: 0.,
            gyr_res: 0.,
        }
    }

    pub fn init(&mut self) {
        println!("chip_id: 0x{:x}", self.imu.get_chip_id().unwrap());

        self.reset();

        self.imu.init(&bmi270::config::BMI260_CONFIG_FILE).unwrap();

        let acc_range = self.acc_range();
        let gyr_range = self.gyr_range();

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
        self.imu.set_pwr_ctrl(pwr_ctrl).unwrap();
    }

    fn reset(&mut self) {
        self.imu.send_cmd(bmi270::Cmd::SoftReset);
        sleep(Duration::from_millis(10));
    }

    pub fn acc_range(&mut self) -> u8 {
        // [g] +/- range (i.e., half of span)
        1 << (1 + self.imu.get_acc_range().unwrap() as u8)
    }

    pub fn gyr_range(&mut self) -> u16 {
        // [deg/s] +/- range (i.e., half of span)
        2000 >> (self.imu.get_gyr_range().unwrap().range as u8)
    }

    pub fn data(&mut self) -> (Vec<f32>, Vec<f32>, u32) {
        let data = self.imu.get_data().unwrap();
        let a: Vec<f32> = {
            vec![data.acc.x, data.acc.y, data.acc.z]
                .iter()
                .map(|x| (*x as f32) * self.acc_res)
                .collect()
        };
        let g: Vec<f32> = {
            vec![data.gyr.x, data.gyr.y, data.gyr.z]
                .iter()
                .map(|x| (*x as f32) * self.gyr_res)
                .collect()
        };
        let t = data.time;
        (a, g, t)
    }

    pub fn dt(t0: u32, t1: u32) -> f32 {
        IMU::<BMI260>::SEC_PER_TICK * (t1.wrapping_sub(t0) as f32)
    }
}

impl IMU<BMI160> {
    const SEC_PER_TICK: f32 = 39e-6; // [s/tick]
    const BITMASK_24: u32 = 0xffffff;
    const SENSEL: bmi160::SensorSelector = bmi160::SensorSelector::new().accel().gyro().time();

    pub fn new(i2c_dev: String, i2c_addr: u8) -> Self {
        IMU::<BMI160> {
            imu: bmi160::Bmi160::new_with_i2c(
                hal::I2cdev::new(i2c_dev.as_str()).unwrap(),
                match i2c_addr {
                    0x68 => bmi160::SlaveAddr::default(),
                    0x69 => bmi160::SlaveAddr::Alternative(true),
                    _ => panic!("Invalid address: {}", i2c_addr),
                },
            ),
            acc_res: 0.,
            gyr_res: 0.,
        }
    }

    pub fn init(&mut self) {
        println!("chip_id: 0x{:x}", self.imu.chip_id().unwrap());

        self.reset();

        // occasionally, first attempt doesn't take
        for _ in 0..2 {
            self.imu
                .set_accel_power_mode(bmi160::AccelerometerPowerMode::Normal);
            self.imu
                .set_gyro_power_mode(bmi160::GyroscopePowerMode::Normal);
        }

        self.acc_res = 4.0 / (u16::MAX as f32); // [g/bit] resolution
        self.gyr_res = 3000.0 / (u16::MAX as f32); // [deg/s/bit] resolution
    }

    fn reset(&mut self) {
        //TODO: write 0xB6 to 0x7E
        sleep(Duration::from_millis(10));
    }

    pub fn data(&mut self) -> (Vec<f32>, Vec<f32>, u32) {
        let data = self.imu.data(IMU::<BMI160>::SENSEL).unwrap();
        let a: Vec<f32> = {
            let a = data.accel.unwrap();
            vec![a.x, a.y, a.z]
                .iter()
                .map(|x| (*x as f32) * self.acc_res)
                .collect()
        };
        let g: Vec<f32> = {
            let g = data.gyro.unwrap();
            vec![g.x, g.y, g.z]
                .iter()
                .map(|x| (*x as f32) * self.gyr_res)
                .collect()
        };
        let t = data.time.unwrap();
        (a, g, t)
    }

    pub fn dt(t0: u32, t1: u32) -> f32 {
        IMU::<BMI160>::SEC_PER_TICK * ((IMU::<BMI160>::BITMASK_24 & t1.wrapping_sub(t0)) as f32)
    }
}
