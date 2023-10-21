#[cfg(any(feature = "bmi160", feature = "default"))]
pub mod bmi160;

#[cfg(any(feature = "bmi260", feature = "default"))]
pub mod bmi260;

// pub mod imu;
use anyhow::Error;
use std::ops::Mul;
use thiserror::Error;
#[cfg(feature = "async")]
use tokio::sync::mpsc::Sender;

#[derive(Error, Debug)]
pub enum IMUError {
    #[error("failed to initialize")]
    Initialization,
    #[error("failed to communicate")]
    Driver,
    #[error("unknown IMU error")]
    Unknown,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TriAx<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Copy + Clone> From<[T; 3]> for TriAx<T> {
    fn from(a: [T; 3]) -> Self {
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
        }
    }
}

impl<T: Copy + Clone> From<&[T; 3]> for TriAx<T> {
    fn from(a: &[T; 3]) -> Self {
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
        }
    }
}

impl<T: Mul<Output = T> + Copy> Mul<T> for &TriAx<T> {
    type Output = TriAx<T>;
    fn mul(self, rhs: T) -> Self::Output {
        TriAx {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<T: Mul<Output = T> + Copy> Mul<T> for &'static mut TriAx<T> {
    type Output = &'static mut TriAx<T>;
    fn mul(self, rhs: T) -> Self::Output {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
        self.z = self.z * rhs;
        self
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

pub trait IMU {
    fn new(i2c_dev: &str, i2c_addr: u8) -> Result<Self, IMUError>
    where
        Self: Sized;
    fn init(&mut self) -> Result<(), IMUError>;
    fn data(&mut self) -> Result<Data<f32, f32>, IMUError>;
}

#[derive(Debug, Default)]
pub struct BMI<T> {
    pub drv: T,
    pub acc_res: f32,
    pub gyr_res: f32,
    pub t: u32,
}

// pub enum IMUs {
//     BMI160,
//     BMI260,
// }

#[cfg(feature = "async")]
#[async_trait]
trait Sender<T, U> {
    async fn sender(&mut self, tx: Sender<Data<T, U>>, interval: Duration) {
        loop {
            tokio::time::sleep(interval).await;
            tx.send(self.data()).await.unwrap();
        }
    }
}
