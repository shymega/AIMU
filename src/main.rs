#![allow(unused)]
#![allow(dead_code)]

extern crate linux_embedded_hal as hal;
use evdev::{
    self, uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent, RelativeAxisType,
};

mod config;
mod imu;
mod motion;
use config::ConfigAIMU;
use imu::{IMUError, BMI, IMU};

use std::{error::Error, thread::sleep, time::Duration};

#[cfg(feature = "default")]
fn imu_selector(cfg: &ConfigAIMU) -> anyhow::Result<Box<dyn IMU>> {
    Ok(match cfg.imu.model.as_str() {
        "bmi160" => Box::new(BMI::<imu::bmi160::BMI160I2C>::new(
            &cfg.imu.i2c_dev,
            cfg.imu.i2c_addr,
        )?),
        "bmi260" => Box::new(BMI::<imu::bmi260::BMI260I2C>::new(
            &cfg.imu.i2c_dev,
            cfg.imu.i2c_addr,
        )?),
        _ => panic!("Invalid model."),
    })
}

fn main() -> anyhow::Result<()> {
    #[cfg(not(feature = "cli"))]
    let cfg = ConfigAIMU::default();
    #[cfg(feature = "cli")]
    let cfg = ConfigAIMU::from_cli()?;

    //TODO: implement runtime switch for selecting frame based on cfg.user.frame
    // let mut motion = motion::Motion<motion::Frame::Local>::new(cfg.user.scale, cfg.device.screen);
    let mut motion = motion::Motion::new(cfg.user.scale, cfg.device.screen, motion::Frame::Local);

    #[cfg(all(feature = "bmi160", not(feature = "default")))]
    let mut imu: BMI<imu::bmi160::BMI160I2C> = IMU::new(&cfg.imu.i2c_dev, cfg.imu.i2c_addr)?;
    #[cfg(all(feature = "bmi260", not(feature = "default")))]
    let mut imu: BMI<imu::bmi260::BMI260I2C> = IMU::new(&cfg.imu.i2c_dev, cfg.imu.i2c_addr)?;
    #[cfg(feature = "default")]
    let mut imu = &mut *imu_selector(&cfg)?;
    imu.init()?;

    let mut dev_vr = VirtualDeviceBuilder::new()?
        .name("AIMU")
        .with_relative_axes(&AttributeSet::from_iter([
            RelativeAxisType::REL_X,
            RelativeAxisType::REL_Y,
            RelativeAxisType::REL_WHEEL, // convinces libinput it's a mouse
        ]))?
        .build()?;

    let update_interval = Duration::from_micros((1e6 / cfg.user.freq) as u64);

    loop {
        let data = imu.data()?;
        let xy_mot = motion.process(data.a.into(), data.g.into(), data.t);
        dev_vr.emit(&[
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_X.0, xy_mot.x),
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_Y.0, xy_mot.y),
        ])?;
        sleep(update_interval);
    }
    Ok(())
}
