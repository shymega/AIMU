# AIMU

- Userspace IMU-assisted aiming for Linux.
- Creates virtual mouse.
- Maps accelerometer+gyroscope motion to virtual mouse movements.
- Accounts for angle between screen and keyboard (configurable).

## Usage
1. Disable `bmi160_i2c` and `bmi160_core` kernel modules.
1. Enable `i2c_dev` kernel module.
1. Add user to `i2c` group.
1. Tweak `gyr_scale`, `update_freq`, `scr_angle` in `src/config.rs` to taste.
1. ```shell
   cargo run --features bmi160
   # or
   cargo run --features bmi260
   ```

## TODO
- Refine virtual gamepad/joystick (separate branch)
- Configuration file and argument parsing

## Aknowledgements
- [https://github.com/JibbSmart/GamepadMotionHelpers](https://github.com/JibbSmart/GamepadMotionHelpers)
- [https://github.com/qrasmont/bmi270](https://github.com/qrasmont/bmi270)
