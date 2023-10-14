#![allow(unused)]
#![allow(dead_code)]

extern crate linux_embedded_hal as hal;
use evdev::{
    self, uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent, RelativeAxisType,
};

mod config;
mod imu;
#[cfg(feature = "bmi160")]
mod imu_bmi160;
#[cfg(feature = "bmi260")]
mod imu_bmi260;
mod motion;
use imu::{IMUError, BMI, IMU};

use std::{error::Error, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    //let cfg = config::ConfigAIMU::default();
    let cfg = config::ConfigAIMU::from_cli()?;

    //TODO: implement runtime switch for selecting frame based on cfg.user.frame
    // let mut motion = motion::Motion<motion::Frame::Local>::new(cfg.user.scale, cfg.device.screen);
    let mut motion = motion::Motion::new(cfg.user.scale, cfg.device.screen);

    //FIXME: implement compiletime switch
    #[cfg(feature = "bmi160")]
    let mut imu: BMI<imu_bmi160::BMI160I2C> = imu::IMU::new(&cfg.imu.i2c_dev, cfg.imu.i2c_addr);
    #[cfg(feature = "bmi260")]
    let mut imu: BMI<imu_bmi260::BMI260I2C> = imu::IMU::new(&cfg.imu.i2c_dev, cfg.imu.i2c_addr);
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
