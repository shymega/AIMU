#![allow(clippy::too_many_arguments)]
#[allow(unused)]
use autocxx::prelude::*;
use glam::IVec2;
use std::pin::Pin;

include_cpp! {
    #include "GamepadMotion.hpp"
    generate!("GamepadMotion")
    safety!(unsafe_ffi)
}

pub enum Frame {
    Local,
    Player,
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

    pub fn process(&mut self, a: [f32; 3], g: [f32; 3], dt: f32) -> IVec2 {
        // FIXME: is there a more elegant way to unpack arrays?
        self.motion
            .pin_mut()
            .ProcessMotion(g[0], g[1], g[2], a[0], a[1], a[2], dt);
        self.frame(dt)
    }

    //FIXME: select frame using generics
    fn frame(&mut self, dt: f32) -> IVec2 {
        self.frame_local(dt)
    }

    fn frame_local(&mut self, dt: f32) -> IVec2 {
        let (mut gx, mut gy, mut gz): (f32, f32, f32) = (0.0, 0.0, 0.0);
        self.motion.pin_mut().GetCalibratedGyro(
            Pin::new(&mut gx),
            Pin::new(&mut gy),
            Pin::new(&mut gz),
        );
        // let x = ((gx * self.sincos.1 - (-gz) * self.sincos.0) * self.scale * dt) as i32;
        // let y = ((-gy) * self.scale * dt) as i32;
        // let y = ((gy * sincos.0 - gz * sincos.1) * -scale * dt) as i32;
        IVec2::new(
            ((gx * self.sincos.1 - (-gz) * self.sincos.0) * self.scale * dt) as i32,
            ((-gy) * self.scale * dt) as i32,
        )
    }

    #[allow(unused_mut)]
    #[allow(unused_variables)]
    fn frame_player(&mut self, dt: f32) -> IVec2 {
        let (mut x, mut y, mut z): (f32, f32, f32) = (0.0, 0.0, 0.0);
        self.motion
            .pin_mut()
            .GetPlayerSpaceGyro(Pin::new(&mut x), Pin::new(&mut y), 1.41);
        IVec2::new((x * self.scale) as i32, (y * self.scale) as i32)
    }
}
