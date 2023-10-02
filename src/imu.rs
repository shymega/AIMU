extern crate linux_embedded_hal as hal;
// use hal::i2cdev::core::I2CDevice;
use std::{ops::Mul, thread::sleep, time::Duration};
use tokio::sync::mpsc::Sender;

// #[cfg(feature = "bmi160")]
use bmi160;

// #[cfg(not(feature = "bmi160"))]
use bmi270;

#[derive(Copy, Clone, Debug, Default)]
pub struct TriAx<T> {
    x: T,
    y: T,
    z: T,
}

impl<T> Mul<T> for &TriAx<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = TriAx<T>;
    fn mul(self, rhs: T) -> Self::Output {
        TriAx {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<T> From<bmi270::AxisData> for TriAx<T>
where
    T: From<i16>,
{
    fn from(d: bmi270::AxisData) -> Self {
        Self {
            x: <T>::from(d.x),
            y: <T>::from(d.y),
            z: <T>::from(d.z),
        }
    }
}

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

impl<T> Into<[T; 3]> for TriAx<T> {
    fn into(self) -> [T; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Data<T, U> {
    pub a: TriAx<T>,
    pub g: TriAx<T>,
    pub t: U,
}

impl<T, U> From<bmi270::Data> for Data<T, U>
where
    T: From<i16>,
    U: From<u32>,
{
    fn from(d: bmi270::Data) -> Self {
        Self {
            a: TriAx::<T>::from(d.acc),
            g: TriAx::<T>::from(d.gyr),
            t: <U>::from(d.time),
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

#[derive(Debug, Default)]
pub struct BMI<T> {
    drv: T,
    acc_res: f32,
    gyr_res: f32,
    t: u32,
}

// #[cfg(feature = "bmi160")]
pub type BMI160I2C = bmi160::Bmi160<bmi160::interface::I2cInterface<hal::I2cdev>>;
// #[cfg(not(feature = "bmi160"))]
pub type BMI260I2C = bmi270::Bmi270<bmi270::interface::I2cInterface<hal::I2cdev>>;

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

pub struct BMI260 {
    drv: BMI260I2C,
    acc_res: f32,
    gyr_res: f32,
    t: u32,
}

impl BMI260 {
    const SEC_PER_TICK: f32 = 39e-6; // [s/tick]

    fn reset(&mut self) {
        self.drv.send_cmd(bmi270::Cmd::SoftReset).unwrap();
        sleep(Duration::from_millis(10));
    }

    fn acc_range(&mut self) -> u8 {
        // [g] +/- range (i.e., half of span)
        1 << (1 + self.drv.get_acc_range().unwrap() as u8)
    }

    fn gyr_range(&mut self) -> u16 {
        // [deg/s] +/- range (i.e., half of span)
        2000 >> (self.drv.get_gyr_range().unwrap().range as u8)
    }

    async fn sender(&mut self, tx: Sender<Data<f32, f32>>, interval: Duration) {
        loop {
            tokio::time::sleep(interval).await;
            tx.send(self.data()).await.unwrap();
        }
    }
}

pub enum IMUs {
    BMI160(BMI160),
    BMI260(BMI260),
}

pub trait IMU {
    fn new(i2c_dev: String, i2c_addr: u8) -> Self;
    fn init(&mut self);
    fn data(&mut self) -> Data<f32, f32>;
    fn dt(&self, t: u32) -> f32;
}

// #[cfg(not(feature = "bmi160"))]
impl IMU for BMI260 {
    fn new(i2c_dev: String, i2c_addr: u8) -> Self {
        BMI260 {
            drv: bmi270::Bmi270::new_i2c(
                hal::I2cdev::new(i2c_dev.as_str()).unwrap(),
                match i2c_addr {
                    0x68 => bmi270::I2cAddr::Default,
                    0x69 => bmi270::I2cAddr::Alternative,
                    _ => panic!("Invalid address: {}", i2c_addr),
                },
                bmi270::Burst::Other(255),
            ),
            t: 0,
            acc_res: 0.,
            gyr_res: 0.,
        }
    }

    fn init(&mut self) {
        println!("chip_id: 0x{:x}", self.drv.get_chip_id().unwrap());

        self.reset();

        self.drv.init(&bmi270::config::BMI260_CONFIG_FILE).unwrap();

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
        self.drv.set_pwr_ctrl(pwr_ctrl).unwrap();
    }

    fn data(&mut self) -> Data<f32, f32> {
        let mut data = Data::from(self.drv.get_data().unwrap());
        let dt: f32 = self.dt(data.t);
        self.t = data.t;
        Data {
            a: &data.a * self.acc_res,
            g: &data.g * self.gyr_res,
            t: dt,
        }
    }

    fn dt(&self, t: u32) -> f32 {
        Self::SEC_PER_TICK * (t.wrapping_sub(self.t) as f32)
    }
}

// #[cfg(feature = "bmi160")]
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
