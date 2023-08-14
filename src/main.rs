#![allow(unused)]
#![allow(dead_code)]

extern crate linux_embedded_hal as hal;
use autocxx::prelude::*;
use evdev::{
    uinput::VirtualDevice, uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent,
    InputId, RelativeAxisType,
};
mod imu;

use std::{error::Error, pin::Pin, thread::sleep, time::Duration};
include_cpp! {
    #include "GamepadMotion.hpp"
    generate!("GamepadMotion")
    safety!(unsafe_ffi)
}

#[derive(Debug)]
struct ConfigAIMU {
    imu: ConfigIMU,
    device: ConfigDevice,
    user: ConfigUser,
}

#[derive(Debug, Default)]
struct ConfigIMU {
    model: String,
    i2c_dev: String,
    i2c_addr: u8,
}

#[derive(Debug, Default)]
struct ConfigDevice {
    screen: f32,
    orient: [u8; 9],
}

#[derive(Debug, Default)]
struct ConfigUser {
    scale: f32,
    freq: f32,
    space: String,
}

impl Default for ConfigAIMU {
    fn default() -> Self {
        Self {
            imu: ConfigIMU {
                model: String::from("bmi260"),
                i2c_dev: String::from("/dev/i2c-2"),
                i2c_addr: 0x69,
            },
            device: ConfigDevice {
                /// [deg] angle between keyboard and screen
                screen: 135.,
                /// orientation array [xx, xy, xz, yx, yy, yz, zx, zy, zz]
                orient: [1, 0, 0, 0, 1, 0, 0, 0, 1],
            },
            user: ConfigUser {
                /// [-] arbitrary scale factor
                scale: 30.0,
                /// [Hz] update frequency
                freq: 40.0,
                /// frame of reference for processing motion control
                space: String::from("local"),
            },
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = ConfigAIMU::default();
    let update_interval = Duration::from_micros((1e6 / cfg.user.freq) as u64);
    let sincos = ((cfg.device.screen - 90.) * std::f32::consts::PI / 180.).sin_cos();

    let mut motion = ffi::GamepadMotion::new().within_unique_ptr();

    let motion_space = {
        match cfg.user.space.as_str() {
            "local" => local_space,
            "player" => player_space,
            _ => panic!("Unsupported motion space: {}", cfg.user.space),
        }
    };

    // let mut imu = match cfg.imu.model.as_str() {
    // "bmi160" => imu::IMU::<imu::BMI160>::new(cfg.imu.i2c_dev, cfg.imu.i2c_addr),
    // "bmi260" => imu::IMU::<imu::BMI260>::new(cfg.imu.i2c_dev, cfg.imu.i2c_addr),
    // _ => panic!("Unsupported motion space: {}", cfg.user.space),
    // };
    let mut imu = imu::IMU::<imu::BMI260>::new(cfg.imu.i2c_dev, cfg.imu.i2c_addr);
    imu.init();

    let mut vdev = VirtualDeviceBuilder::new()?
        .name("aimu")
        .with_relative_axes(&AttributeSet::from_iter([
            RelativeAxisType::REL_X,
            RelativeAxisType::REL_Y,
            RelativeAxisType::REL_WHEEL, // convince libinput
        ]))?
        .build()?;
    for path in vdev.enumerate_dev_nodes_blocking()? {
        let path = path?;
        println!("vdev: {}", path.display());
    }

    let mut t_pre: u32 = imu.data().2;

    loop {
        let (a, g, t) = imu.data();
        let dt = imu.dt(t_pre, t);
        t_pre = t;
        // println!(
        //     "a: {}\t{}\t{}\ng: {}\t{}\t{}\nt: {}\tdt: {}",
        //     a[0], a[1], a[2], g[0], g[1], g[2], t, dt
        // );
        // FIXME: is there a more elegant way to unpack arrays?
        // TODO: BMI160 read order is ax,ay,az,gx,gy,gz - handle reversed g[],a[] order for compatibility.
        motion.pin_mut().ProcessMotion(
            g[0].into(),
            g[1].into(),
            g[2].into(),
            a[0].into(),
            a[1].into(),
            a[2].into(),
            dt,
        );

        let (x, y) = motion_space(&mut motion, &sincos, dt, cfg.user.scale);
        // dbg!("x: {:5}\ty: {:5}", x, y);

        vdev.emit(&[
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_X.0, x),
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_Y.0, y),
        ])?;
        sleep(update_interval);
    }
    Ok(())
}

fn local_space(
    motion: &mut UniquePtr<ffi::GamepadMotion>,
    sincos: &(f32, f32),
    dt: f32,
    scale: f32,
) -> (i32, i32) {
    let (mut gx, mut gy, mut gz): (f32, f32, f32) = (0.0, 0.0, 0.0);
    motion
        .pin_mut()
        .GetCalibratedGyro(Pin::new(&mut gx), Pin::new(&mut gy), Pin::new(&mut gz));
    let x = ((gx * sincos.1 - (-gz) * sincos.0) * scale * dt) as i32;
    let y = ((-gy) * scale * dt) as i32;
    //let y = ((gy * sc.0 - gz * sc.1) * -scale * dt) as i32;
    (x, y)
}

fn player_space(
    motion: &mut UniquePtr<ffi::GamepadMotion>,
    sincos: &(f32, f32),
    dt: f32,
    scale: f32,
) -> (i32, i32) {
    let (mut x, mut y, mut z): (f32, f32, f32) = (0.0, 0.0, 0.0);
    motion
        .pin_mut()
        .GetPlayerSpaceGyro(Pin::new(&mut x), Pin::new(&mut y), 1.41);
    ((x * scale) as i32, (y * scale) as i32)
}
