#[derive(Debug)]
pub struct ConfigAIMU {
    pub imu: ConfigIMU,
    pub device: ConfigDevice,
    pub user: ConfigUser,
}

#[derive(Debug, Default)]
pub struct ConfigIMU {
    pub model: String,
    pub i2c_dev: String,
    pub i2c_addr: u8,
}

#[derive(Debug, Default)]
pub struct ConfigDevice {
    pub screen: f32,
    pub orient: [u8; 9],
}

#[derive(Debug, Default)]
pub struct ConfigUser {
    pub scale: f32,
    pub freq: f32,
    pub frame: String,
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
                frame: String::from("local"),
            },
        }
    }
}
