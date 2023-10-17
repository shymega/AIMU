#![allow(unused)]
#![allow(dead_code)]

use autocxx::prelude::*;
use std::pin::Pin;

include_cpp! {
    #include "GamepadMotion.hpp"
    safety!(unsafe_ffi)
    generate!("GamepadMotion")
}

pub enum Frame {
    Local,
    Player,
}

#[derive(Debug, Default)]
pub struct BiAx<T> {
    pub x: T,
    pub y: T,
}

pub struct Motion {
    motion: UniquePtr<ffi::GamepadMotion>,
    scale: f32,
    sincos: (f32, f32),
    frame: Frame,
}

impl Motion {
    pub fn new(scale: f32, screen: f32, frame: Frame) -> Self {
        Self {
            motion: ffi::GamepadMotion::new().within_unique_ptr(),
            scale,
            sincos: screen.to_radians().sin_cos(),
            frame,
        }
    }

    pub fn process(&mut self, a: [f32; 3], g: [f32; 3], dt: f32) -> BiAx<i32> {
        // FIXME: is there a more elegant way to unpack arrays?
        self.motion.pin_mut().ProcessMotion(
            g[0].into(),
            g[1].into(),
            g[2].into(),
            a[0].into(),
            a[1].into(),
            a[2].into(),
            dt,
        );
        self.frame(dt)
    }

    //FIXME: select frame using generics
    fn frame(&mut self, dt: f32) -> BiAx<i32> {
        self.frame_local(dt)
    }

    fn frame_local(&mut self, dt: f32) -> BiAx<i32> {
        let (mut gx, mut gy, mut gz): (f32, f32, f32) = (0.0, 0.0, 0.0);
        self.motion.pin_mut().GetCalibratedGyro(
            Pin::new(&mut gx),
            Pin::new(&mut gy),
            Pin::new(&mut gz),
        );
        // let x = ((gx * self.sincos.1 - (-gz) * self.sincos.0) * self.scale * dt) as i32;
        // let y = ((-gy) * self.scale * dt) as i32;
        // let y = ((gy * sincos.0 - gz * sincos.1) * -scale * dt) as i32;
        BiAx::<i32> {
            x: ((gx * self.sincos.1 - (-gz) * self.sincos.0) * self.scale * dt) as i32,
            y: ((-gy) * self.scale * dt) as i32,
        }
    }

    fn frame_player(&mut self, dt: f32) -> BiAx<i32> {
        let (mut x, mut y, mut z): (f32, f32, f32) = (0.0, 0.0, 0.0);
        self.motion
            .pin_mut()
            .GetPlayerSpaceGyro(Pin::new(&mut x), Pin::new(&mut y), 1.41);
        BiAx::<i32> {
            x: (x * self.scale) as i32,
            y: (y * self.scale) as i32,
        }
    }
}
