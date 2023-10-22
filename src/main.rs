// #![allow(unused)]
#![allow(dead_code)]

extern crate linux_embedded_hal as hal;

mod config;
mod device;
mod imu;
mod motion;
use anyhow::Result;
use config::ConfigAIMU;
use device::{trigger::Trigger, vmouse::VMouse};
use evdev;
use imu::{BMI, IMU};
use std::{thread::sleep, time::Duration};

#[cfg(feature = "default")]
fn imu_selector(cfg: &ConfigAIMU) -> Result<Box<dyn IMU>> {
    Ok(match cfg.imu.model {
        Some(imu::IMUs::BMI160) => Box::new(BMI::<imu::bmi160::BMI160I2C>::new(
            &cfg.imu.i2c_dev,
            cfg.imu.i2c_addr,
        )?),
        Some(imu::IMUs::BMI260) => Box::new(BMI::<imu::bmi260::BMI260I2C>::new(
            &cfg.imu.i2c_dev,
            cfg.imu.i2c_addr,
        )?),
        None => panic!("No IMU model specified."),
    })
}

fn main() -> Result<()> {
    #[cfg(not(feature = "cli"))]
    let cfg = ConfigAIMU::default();
    #[cfg(feature = "cli")]
    let cfg = ConfigAIMU::from_cli();

    //TODO: implement runtime switch for selecting frame based on cfg.user.frame
    // let mut motion = motion::Motion<motion::Frame::Local>::new(cfg.user.scale, cfg.device.screen);
    let mut motion = motion::Motion::new(cfg.user.scale, cfg.device.screen, motion::Frame::Local);

    #[cfg(all(feature = "bmi160", not(feature = "default")))]
    let mut imu: BMI<imu::bmi160::BMI160I2C> = IMU::new(&cfg.imu.i2c_dev, cfg.imu.i2c_addr)?;
    #[cfg(all(feature = "bmi260", not(feature = "default")))]
    let mut imu: BMI<imu::bmi260::BMI260I2C> = IMU::new(&cfg.imu.i2c_dev, cfg.imu.i2c_addr)?;
    #[cfg(feature = "default")]
    let imu = &mut *imu_selector(&cfg)?;
    imu.init()?;

    let mut vdev = VMouse::new()?;
    let trig = Trigger::new(
        "Microsoft X-Box 360 pad",
        evdev::InputEventKind::AbsAxis(evdev::AbsoluteAxisType::ABS_Z),
        10,
    );
    let update_interval = Duration::from_micros((1e6 / cfg.user.freq) as u64);

    trig.task();

    loop {
        let data = imu.data()?;
        let xy_mot = motion.process(data.a.into(), data.g.into(), data.t);
        if trig.check() {
            vdev.update(xy_mot.x, xy_mot.y)?;
        }
        sleep(update_interval);
    }
}
