#![allow(unused)]
#![allow(dead_code)]

extern crate linux_embedded_hal as hal;
use evdev::{
    self,
    uinput::{VirtualDevice, VirtualDeviceBuilder, VirtualEventStream},
    AbsInfo, AbsoluteAxisType, AttributeSet, Device, EventStream, EventType, InputEvent,
    InputEventKind, InputId, RelativeAxisType, UinputAbsSetup,
};
use tokio::sync::mpsc::Sender;

mod config;
mod imu;
mod motion;

use std::{error::Error, pin::Pin, thread::sleep, time::Duration};

async fn dev_event_sender(tx: Sender<InputEvent>, dev: Device) {
    let mut events = dev.into_event_stream().unwrap();
    loop {
        let ev = events.next_event().await.unwrap();
        tx.send(ev).await.unwrap();
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = config::ConfigAIMU::default();

    //TODO: implement runtime switch for selecting frame based on cfg.user.frame
    // let mut motion = motion::Motion<motion::Frame::Local>::new(cfg.user.scale, cfg.device.screen);
    let mut motion = motion::Motion::new(cfg.user.scale, cfg.device.screen);

    //FIXME: implement runtime switch...pref. without Box<>
    // #[cfg(feature = "bmi160")]
    // let mut imu = imu::IMU::<imu::BMI160I2C>::new(cfg.imu.i2c_dev, cfg.imu.i2c_addr);
    // #[cfg(not(feature = "bmi160"))]
    // let mut imu = imu::IMU::<imu::BMI260I2C>::new(cfg.imu.i2c_dev, cfg.imu.i2c_addr);
    let mut imu = imu::IMU::<imu::BMI260I2C>::new(cfg.imu.i2c_dev, cfg.imu.i2c_addr);
    imu.init();

    let mut dev_hw = {
        let mut devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
        devices.reverse();
        devices.into_iter().nth(0).unwrap()
    };
    println!("{dev_hw}");
    dev_hw.grab();
    let mut dev_vr = {
        let mut dev_vr = VirtualDeviceBuilder::new()?
            .name("AIMU")
            // .name("Microsoft X-Box 360 pad")
            .input_id(dev_hw.input_id())
            .with_keys(&dev_hw.supported_keys().unwrap())?;
        // .with_ff(&dev_hw.supported_ff().unwrap())?;
        let abs_ax: Vec<UinputAbsSetup> = dev_hw
            .supported_absolute_axes()
            .unwrap()
            .iter()
            .map(|ax| UinputAbsSetup::new(ax, AbsInfo::new(0, 0, 0, 0, 0, 0)))
            .collect();
        for ax in abs_ax {
            dev_vr = dev_vr.with_absolute_axis(&ax)?
        }
        dev_vr.build()?
    };

    let mut xy_abs = motion::BiAx::<i32> { x: 0, y: 0 };
    let mut xy_mot = motion::BiAx::<i32> { x: 0, y: 0 };
    let (tx0, mut rx0) = tokio::sync::mpsc::channel::<imu::Data<f32, f32>>(1);
    let (tx1, mut rx1) = tokio::sync::mpsc::channel::<InputEvent>(100);

    tokio::task::spawn(async move {
        imu.sender(tx0, Duration::from_micros((1e6 / cfg.user.freq) as u64))
            .await;
    });

    tokio::task::spawn(async move {
        dev_event_sender(tx1, dev_hw).await;
    });

    loop {
        tokio::select! {
            Some(data) = rx0.recv() => {
                xy_mot = motion.process(data.a.into(), data.g.into(), data.t);
                xy_abs.x = xy_abs.x.saturating_add(xy_mot.x);
                xy_abs.y = xy_abs.y.saturating_add(xy_mot.y);
                let evo = [InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_RX.0, xy_abs.x),
                           InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_RY.0, xy_abs.y)];
                dev_vr.emit(&evo)?;
            },
            Some(ev) = rx1.recv() => {
                let evo = match ev.kind() {
                    InputEventKind::AbsAxis(AbsoluteAxisType::ABS_RX) => {
                        xy_abs.x = ev.value().saturating_add(xy_mot.x);
                        xy_mot.x = 0;
                        InputEvent::new(EventType::ABSOLUTE, ev.code(), xy_abs.x)
                    }
                    InputEventKind::AbsAxis(AbsoluteAxisType::ABS_RY) => {
                        xy_abs.y = ev.value().saturating_add(xy_mot.y);
                        xy_mot.y = 0;
                        InputEvent::new(EventType::ABSOLUTE, ev.code(), xy_abs.y)
                    }
                    _ => ev,
                };
                dev_vr.emit(&[evo])?;
            }
        }
    }
    Ok(())
}
