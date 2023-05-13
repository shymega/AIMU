#![allow(unused)]
#![allow(dead_code)]

extern crate linux_embedded_hal as hal;
use autocxx::prelude::*;
use bmi160::{
    AccelerometerPowerMode, Bmi160, GyroscopePowerMode, Sensor3DData, SensorSelector, SlaveAddr,
};
use evdev::{
    uinput::VirtualDevice, uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent,
    InputId, RelativeAxisType,
};
use std::{error::Error, pin::Pin, thread::sleep, time::Duration};
include_cpp! {
    #include "GamepadMotion.hpp"
    generate!("GamepadMotion")
    safety!(unsafe_ffi)
}

const BITMASK_24: u32 = 0xffffff;

fn main() -> Result<(), Box<dyn Error>> {
    let acc_res: f32 = 4.0 / (u16::MAX as f32); // [g/bit] resolution, default range is +/-2 g
    let gyr_res: f32 = 3000.0 / (u16::MAX as f32); // [deg/s/bit] resolution, default range is+/-2000 deg/s
    const IMU_SEC_PER_TICK: f32 = 39e-6; // [s/tick]
    let gyr_scale: f32 = 30.0; // [-] arbitrary scale factor
    let update_freq: f32 = 40.; // [Hz]
    let update_interval = Duration::from_micros((1e6 / update_freq) as u64);
    let scr_angle: f32 = 135.; // [deg] angle between screen and IMU x-y plane
    let sc = ((scr_angle - 90.) * std::f32::consts::PI / 180.).sin_cos();

    let mut motion = ffi::GamepadMotion::new().within_unique_ptr();

    let mut imu = Bmi160::new_with_i2c(
        hal::I2cdev::new("/dev/i2c-2")?,
        SlaveAddr::Alternative(true),
    );
    for i in 0..2 {
        imu.set_accel_power_mode(AccelerometerPowerMode::Normal);
        imu.set_gyro_power_mode(GyroscopePowerMode::Normal);
    }

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

    let sensel = SensorSelector::new().accel().gyro().time();
    let mut t_pre: u32 = imu.data(sensel).unwrap().time.unwrap();

    loop {
        let data = imu.data(sensel).unwrap();
        let a: Vec<f32> = {
            let a = data.accel.unwrap();
            vec![a.x, a.y, a.z]
                .iter()
                .map(|x| (*x as f32) * acc_res)
                .collect()
        };
        let g: Vec<f32> = {
            let g = data.gyro.unwrap();
            vec![g.x, g.y, g.z]
                .iter()
                .map(|x| (*x as f32) * gyr_res)
                .collect()
        };
        let t = data.time.unwrap() as u32;
        let dt = IMU_SEC_PER_TICK * ((BITMASK_24 & t.wrapping_sub(t_pre)) as f32);
        t_pre = t;
        // println!(
        //     "a: {}\t{}\t{}\n\
        //      g: {}\t{}\t{}\n\
        //      t: {}\t dt={}",
        //     a[0], a[1], a[2], g[0], g[1], g[2], t, dt
        // );
        motion.pin_mut().ProcessMotion(
            g[0].into(),
            g[1].into(),
            g[2].into(),
            a[0].into(),
            a[1].into(),
            a[2].into(),
            dt,
        );

        //player space
        // let (mut x, mut y, mut z): (f32, f32, f32) = (0.0, 0.0, 0.0);
        // let xp: Pin<&mut f32> = Pin::new(&mut x);
        // let yp: Pin<&mut f32> = Pin::new(&mut y);
        // let zp: Pin<&mut f32> = Pin::new(&mut z);
        // ////xp.as_mut().set(x);
        // ////yp.as_mut().set(y);
        // motion.pin_mut().GetPlayerSpaceGyro(xp, yp, 1.41);
        // let (x, y) = (x as i32, y as i32);

        //local space
        let (mut gx, mut gy, mut gz): (f32, f32, f32) = (0.0, 0.0, 0.0);
        let gxp: Pin<&mut f32> = Pin::new(&mut gx);
        let gyp: Pin<&mut f32> = Pin::new(&mut gy);
        let gzp: Pin<&mut f32> = Pin::new(&mut gz);
        motion.pin_mut().GetCalibratedGyro(gxp, gyp, gzp);
        // let x = (gx * gyr_scale * dt) as i32;
        let x = ((gx * sc.1 - (-gz) * sc.0) * gyr_scale * dt) as i32;
        let y = ((-gy) * gyr_scale * dt) as i32;
        //let y = ((gy * sc.0 - gz * sc.1) * -gyr_scale * dt) as i32;
        // println!("x={:5} y={:5}", x, y);

        vdev.emit(&[
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_X.0, x),
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_Y.0, y),
        ])?;
        sleep(update_interval);
    }
    Ok(())
}
