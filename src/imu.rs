extern crate linux_embedded_hal as hal;
// use hal::i2cdev::core::I2CDevice;
use std::{ops::Mul, thread::sleep, time::Duration};
use tokio::sync::mpsc::Sender;

// #[cfg(feature = "bmi160")]
// use bmi160;
// #[cfg(feature = "bmi160")]
// pub type BMI160I2C = bmi160::Bmi160<bmi160::interface::I2cInterface<hal::I2cdev>>;

// #[cfg(not(feature = "bmi160"))]
use bmi270;
// #[cfg(not(feature = "bmi160"))]
pub type BMI260I2C = bmi270::Bmi270<bmi270::interface::I2cInterface<hal::I2cdev>>;

pub enum IMUs {
    // BMI160(IMU<BMI160I2C>),
    BMI260(IMU<BMI260I2C>),
}

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

#[derive(Debug, Default)]
pub struct IMU<T> {
    imu: T,
    acc_res: f32,
    gyr_res: f32,
    t: u32,
}

// #[cfg(feature = "bmi160")]
// impl IMU<BMI160I2C> {
//     const SEC_PER_TICK: f32 = 39e-6; // [s/tick]
//     const BITMASK_24: u32 = 0xffffff;

//     pub fn new(i2c_dev: String, i2c_addr: u8) -> Self {
//         IMU::<BMI160I2C> {
//             imu: bmi160::Bmi160::new_with_i2c(
//                 hal::I2cdev::new(i2c_dev.as_str()).unwrap(),
//                 match i2c_addr {
//                     0x68 => bmi160::SlaveAddr::default(),
//                     0x69 => bmi160::SlaveAddr::Alternative(true),
//                     _ => panic!("Invalid address: {}", i2c_addr),
//                 },
//             ),
//             acc_res: 0.,
//             gyr_res: 0.,
//         }
//     }

//     pub fn init(&mut self) {
//         println!("chip_id: 0x{:x}", self.imu.chip_id().unwrap());

//         self.reset();

//         // occasionally, first attempt doesn't take
//         for _ in 0..2 {
//             self.imu
//                 .set_accel_power_mode(bmi160::AccelerometerPowerMode::Normal);
//             self.imu
//                 .set_gyro_power_mode(bmi160::GyroscopePowerMode::Normal);
//         }

//         self.acc_res = 4.0 / (u16::MAX as f32); // [g/bit] resolution
//         self.gyr_res = 3000.0 / (u16::MAX as f32); // [deg/s/bit] resolution
//     }

//     fn reset(&mut self) {
//         todo!(); //TODO: write 0xB6 to 0x7E
//     }

//     pub fn data(&mut self) -> (Vec<f32>, Vec<f32>, u32) {
//         let sensel = bmi160::SensorSelector::new().accel().gyro().time();
//         let data = self.imu.data(sensel).unwrap();
//         let a: Vec<f32> = {
//             let a = data.accel.unwrap();
//             vec![a.x, a.y, a.z]
//                 .iter()
//                 .map(|x| (*x as f32) * self.acc_res)
//                 .collect()
//         };
//         let g: Vec<f32> = {
//             let g = data.gyro.unwrap();
//             vec![g.x, g.y, g.z]
//                 .iter()
//                 .map(|x| (*x as f32) * self.gyr_res)
//                 .collect()
//         };
//         let t = data.time.unwrap();
//         (a, g, t)
//     }

//     pub fn dt(&self, t0: u32, t1: u32) -> f32 {
//         IMU::<BMI160I2C>::SEC_PER_TICK
//             * ((IMU::<BMI160I2C>::BITMASK_24 & t1.wrapping_sub(t0)) as f32)
//     }
// }

// #[cfg(not(feature = "bmi160"))]
impl IMU<BMI260I2C> {
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
            t: 0,
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
        self.imu.send_cmd(bmi270::Cmd::SoftReset).unwrap();
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

    // pub fn data(&mut self) -> (Vec<f32>, Vec<f32>, u32) {
    //     let data = self.imu.get_data().unwrap();
    //     let a: Vec<f32> = {
    //         vec![data.acc.x, data.acc.y, data.acc.z]
    //             .iter()
    //             .map(|x| (*x as f32) * self.acc_res)
    //             .collect()
    //     };
    //     let g: Vec<f32> = {
    //         vec![data.gyr.x, data.gyr.y, data.gyr.z]
    //             .iter()
    //             .map(|x| (*x as f32) * self.gyr_res)
    //             .collect()
    //     };
    //     let t = data.time;
    //     (a, g, t)
    // }

    pub fn data(&mut self) -> Data<f32, f32> {
        let mut data = Data::from(self.imu.get_data().unwrap());
        let dt: f32 = self.dt(data.t);
        self.t = data.t;
        Data {
            a: &data.a * self.acc_res,
            g: &data.g * self.gyr_res,
            t: dt,
        }
    }

    pub fn dt(&self, t: u32) -> f32 {
        IMU::<BMI260I2C>::SEC_PER_TICK * (t.wrapping_sub(self.t) as f32)
    }

    pub async fn sender(&mut self, tx: Sender<Data<f32, f32>>, interval: Duration) {
        loop {
            tokio::time::sleep(interval).await;
            tx.send(self.data()).await.unwrap();
        }
    }
}
