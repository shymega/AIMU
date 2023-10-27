#![allow(dead_code)]

extern crate linux_embedded_hal as hal;

mod config;
mod device;
mod imu;
mod motion;
use anyhow::Result;
use config::Config;
use device::{trigger::Trigger, vmouse::VMouse, VDev};
use imu::IMUs;
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    #[cfg(not(feature = "cli"))]
    let cfg = Config::default();
    #[cfg(feature = "cli")]
    let cfg = Config::from_cli();

    println!("{}", toml::to_string_pretty(&cfg).unwrap());

    //TODO: implement runtime switch for selecting frame based on cfg.user.frame
    // let mut motion = motion::Motion<motion::Frame::Local>::new(cfg.user.scale, cfg.device.screen);
    let mut motion = motion::Motion::new(cfg.device.screen, motion::Frame::Local);

    #[cfg(not(feature = "dynamic"))]
    let mut imu = IMUs::new(&cfg.imu)?;
    #[cfg(feature = "dynamic")]
    let imu = &mut *IMUs::new(&cfg.imu)?;
    imu.init()?;

    let mut vdev = VMouse::new()?;
    let trig = Trigger::new(cfg.device.trigger);
    let update_interval = Duration::from_micros((1e6 / cfg.user.freq) as u64);

    trig.task();

    loop {
        let data = imu.data()?;
        let xy_mot = motion.process(data.g, data.a, data.t, &cfg.user.scale);
        if trig.check() {
            vdev.update(xy_mot.x, xy_mot.y)?;
        }
        sleep(update_interval);
    }
}
