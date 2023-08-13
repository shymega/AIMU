#![allow(unused)]
#![allow(dead_code)]

extern crate linux_embedded_hal as hal;
use autocxx::prelude::*;
use bmi270::{
    config::{BMI160_CONFIG_FILE, BMI260_CONFIG_FILE},
    Bmi270, Burst, I2cAddr, PwrCtrl,
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
                orient: [1, 0, 0, 0, 1, 0, 0, 0, 1],
            },
            user: ConfigUser {
                /// [-] arbitrary scale factor
                scale: 30.0,
                /// [Hz] update frequency
                freq: 40.0,
            },
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = ConfigAIMU::default();
    const IMU_SEC_PER_TICK: f32 = 39.0625e-6; // [s/tick]
    let update_interval = Duration::from_micros((1e6 / cfg.user.freq) as u64);
    let sico = ((cfg.device.screen - 90.) * std::f32::consts::PI / 180.).sin_cos();

    let mut motion = ffi::GamepadMotion::new().within_unique_ptr();

    let mut imu = Bmi270::new_i2c(
        hal::I2cdev::new(cfg.imu.i2c_dev)?,
        match cfg.imu.i2c_addr {
            0x68 => I2cAddr::Default,
            0x69 => I2cAddr::Alternative,
            _ => panic!("Invalid address: {}", cfg.imu.i2c_addr),
        },
        Burst::Other(255),
    );
    println!("chip_id: 0x{:x}", imu.get_chip_id().unwrap());

    match cfg.imu.model.as_str() {
        "bmi160" => imu.init(&BMI160_CONFIG_FILE).unwrap(),
        "bmi260" => imu.init(&BMI260_CONFIG_FILE).unwrap(),
        _ => panic!("Unsupported model: {}", cfg.imu.model),
    };

    let acc_range = 1 << (1 + imu.get_acc_range().unwrap() as u8); // [g] +/- range (i.e., half of span)
    let gyr_range = 2000 >> (imu.get_gyr_range().unwrap().range as u8); // [deg/s] +/- range (i.e., half of span)

    println!("acc_range: ±{} g", acc_range);
    println!("gyr_range:  ±{} °/s", gyr_range);

    let acc_res: f32 = ((acc_range << 1) as f32) / (u16::MAX as f32); // [g/bit] resolution
    let gyr_res: f32 = ((gyr_range << 1) as f32) / (u16::MAX as f32); // [deg/s/bit] resolution

    let pwr_ctrl = PwrCtrl {
        aux_en: false,
        gyr_en: true,
        acc_en: true,
        temp_en: false,
    };
    imu.set_pwr_ctrl(pwr_ctrl).unwrap();

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

    let mut t_pre: u32 = imu.get_sensor_time().unwrap();

    loop {
        let data = imu.get_data().unwrap();
        let a: Vec<f32> = {
            vec![data.acc.x, data.acc.y, data.acc.z]
                .iter()
                .map(|x| (*x as f32) * acc_res)
                .collect()
        };
        let g: Vec<f32> = {
            vec![data.gyr.x, data.gyr.y, data.gyr.z]
                .iter()
                .map(|x| (*x as f32) * gyr_res)
                .collect()
        };
        let t = data.time;
        let dt = IMU_SEC_PER_TICK * (t.wrapping_sub(t_pre) as f32);
        t_pre = t;
        // println!(
        //     "a: {}\t{}\t{}\ng: {}\t{}\t{}\nt: {}\tdt: {}",
        //     a[0], a[1], a[2], g[0], g[1], g[2], t, dt
        // );
        // FIXME: is there a more elegant way to unpack arrays?
        motion.pin_mut().ProcessMotion(
            g[0].into(),
            g[1].into(),
            g[2].into(),
            a[0].into(),
            a[1].into(),
            a[2].into(),
            dt,
        );

        // TODO: pull out into user-selectable functions
        // let (x, y) = player_space(&mut motion, cfg.user.scale);
        let (x, y) = local_space(&mut motion, &sico, dt, cfg.user.scale);
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
    sico: &(f32, f32),
    dt: f32,
    scale: f32,
) -> (i32, i32) {
    let (mut gx, mut gy, mut gz): (f32, f32, f32) = (0.0, 0.0, 0.0);
    motion
        .pin_mut()
        .GetCalibratedGyro(Pin::new(&mut gx), Pin::new(&mut gy), Pin::new(&mut gz));
    let x = ((gx * sico.1 - (-gz) * sico.0) * scale * dt) as i32;
    let y = ((-gy) * scale * dt) as i32;
    //let y = ((gy * sc.0 - gz * sc.1) * -scale * dt) as i32;
    (x, y)
}

fn player_space(motion: &mut UniquePtr<ffi::GamepadMotion>, scale: f32) -> (i32, i32) {
    let (mut x, mut y, mut z): (f32, f32, f32) = (0.0, 0.0, 0.0);
    motion
        .pin_mut()
        .GetPlayerSpaceGyro(Pin::new(&mut x), Pin::new(&mut y), 1.41);
    ((x * scale) as i32, (y * scale) as i32)
}
