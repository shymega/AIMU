use std::ops::Mul;

#[derive(Copy, Clone, Debug, Default)]
pub struct TriAx<T> {
    pub x: T,
    pub y: T,
    pub z: T,
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
    fn new(i2c_dev: String, i2c_addr: u8) -> Self;
    fn init(&mut self);
    fn data(&mut self) -> Data<f32, f32>;
    fn dt(&self, t: u32) -> f32;
}

#[derive(Debug, Default)]
pub struct BMI<T> {
    drv: T,
    acc_res: f32,
    gyr_res: f32,
    t: u32,
}

// pub enum IMUs {
//     BMI160(BMI160),
//     BMI260(BMI260),
// }

