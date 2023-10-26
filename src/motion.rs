#![allow(clippy::too_many_arguments)]
use gamepad_motion::GamepadMotion;
use glam::{Mat3, Vec2Swizzles, Vec3};

pub enum Frame {
    Local,
    Player,
}

pub struct Motion {
    motion: GamepadMotion,
    frame: Frame,
    transform: Mat3,
}

impl Motion {
    pub fn new(screen: f32, frame: Frame) -> Self {
        let sincos = screen.to_radians().sin_cos();
        Self {
            motion: GamepadMotion::new(),
            frame,
            transform: Mat3::from_cols(Vec3::X, Vec3::NEG_Y, Vec3::NEG_Z)
                .transpose()
                .mul_mat3(
                    &Mat3::from_cols_array(&[sincos.1, 0., -sincos.0, 0., 1., 0., 0., 0., 0.])
                        .transpose(),
                ),
        }
    }

    fn transform(&self, g: &Vec3, dt: f32, scale: f32) -> IVec2 {
        self.transform
            .mul_scalar(self.scale * dt)
            .mul_vec3(g)
            .xy
            .as_ivec2()
        // IVec2::new(
        //     ((g.x * self.sincos.1 - (-g.z) * self.sincos.0) * self.scale * dt) as i32,
        //     ((-g.y) * self.scale * dt) as i32,
        // )
    }

    pub fn process(&mut self, a: [f32; 3], g: [f32; 3], dt: f32, scale: &f32) -> IVec2 {
        // FIXME: is there a more elegant way to unpack arrays?
        let xyz = self.motion.process(&g, &a, &dt).gyro_player_space(None);
        self.transform(xyz, dt, scale)
    }

    // pub fn process(&mut self, a: [f32; 3], g: [f32; 3], dt: f32) -> IVec2 {
    //     // FIXME: is there a more elegant way to unpack arrays?
    //     self.motion
    //         .pin_mut()
    //         .ProcessMotion(g[0], g[1], g[2], a[0], a[1], a[2], dt);
    //     self.frame(dt)
    // }

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
