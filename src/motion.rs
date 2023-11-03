#![allow(clippy::too_many_arguments)]
use gamepad_motion::GamepadMotion;
use glam::{swizzles::*, IVec2, Mat3, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
#[cfg(feature = "cli")]
#[derive(clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum Frame {
    #[default]
    Direct,
    Local,
    Player,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// [-] arbitrary scale factor
    pub scale: f32,
    /// [Hz] update frequency
    pub freq: f32,
    /// motion processor frame of reference
    pub frame: Frame,
    /// [deg] acute angle between plane of keyboard and rear of screen
    pub screen: f32,
    /// orientation array [xx, xy, xz, yx, yy, yz, zx, zy, zz]
    pub orient: Mat3,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scale: 50.,
            freq: 40.,
            frame: Frame::default(),
            screen: 45.,
            orient: Mat3::from_cols_array(&[1., 0., 0., 0., -1., 0., 0., 0., -1.]).transpose(),
        }
    }
}

pub struct Motion {
    motion: GamepadMotion,
    frame: Frame,
    transform: Mat3,
}

impl Motion {
    pub fn new(cfg: &Config) -> Self {
        let sincos = cfg.screen.to_radians().sin_cos();
        Self {
            motion: GamepadMotion::new(),
            frame: cfg.frame,
            transform: Mat3::from_cols(Vec3::X, Vec3::NEG_Y, Vec3::NEG_Z)
                .transpose()
                .mul_mat3(
                    &Mat3::from_cols_array(&[sincos.1, 0., sincos.0, 0., 1., 0., 0., 0., 0.])
                        .transpose(),
                ),
        }
    }

    fn transformation(&self, g: Vec3, dt: &f32, scale: f32) -> IVec2 {
        self.transform
            .mul_scalar(scale * dt)
            .mul_vec3(g)
            .xy()
            .as_ivec2()
    }

    pub fn process(&mut self, g: Vec3, a: Vec3, dt: f32, scale: &f32) -> IVec2 {
        let xyz = Vec3::from_array({
            self.motion
                .process(g.to_array(), a.to_array(), &dt)
                .gyro_calibrated()
        });
        self.transformation(xyz, &dt, *scale)
    }

    // fn frame_local(&mut self, dt: f32) -> IVec2 {
    //     let (mut gx, mut gy, mut gz): (f32, f32, f32) = (0.0, 0.0, 0.0);
    //     self.motion.pin_mut().GetCalibratedGyro(
    //         Pin::new(&mut gx),
    //         Pin::new(&mut gy),
    //         Pin::new(&mut gz),
    //     );
    //     // let x = ((gx * self.sincos.1 - (-gz) * self.sincos.0) * self.scale * dt) as i32;
    //     // let y = ((-gy) * self.scale * dt) as i32;
    //     // let y = ((gy * sincos.0 - gz * sincos.1) * -scale * dt) as i32;
    //     IVec2::new(
    //         ((gx * self.sincos.1 - (-gz) * self.sincos.0) * self.scale * dt) as i32,
    //         ((-gy) * self.scale * dt) as i32,
    //     )
    // }

    // #[allow(unused_mut)]
    // #[allow(unused_variables)]
    // fn frame_player(&mut self, dt: f32) -> IVec2 {
    //     let (mut x, mut y, mut z): (f32, f32, f32) = (0.0, 0.0, 0.0);
    //     self.motion
    //         .pin_mut()
    //         .GetPlayerSpaceGyro(Pin::new(&mut x), Pin::new(&mut y), 1.41);
    //     IVec2::new((x * self.scale) as i32, (y * self.scale) as i32)
    // }
}
