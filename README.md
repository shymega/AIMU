# AIMU

- Userspace IMU-assisted aiming for Linux.
- Maps gyro-motion to virtual mouse movements.
- Accounts for angle between screen and keyboard (configurable in code).

## Win Max 2 branch support
|2022 / BMI160| 2023 / BMI260|
|---|---|
|`master` |`bmi260`|

## Usage
1. Disable `bmi160_i2c` and `bmi160_core` kernel modules.
1. Enable `i2c_dev` kernel module.
1. Add user to `i2c` group.
1. Tweak `gyr_scale`, `update_freq`, `scr_angle` to taste.
1. ```shell
   cargo build
   cargo run
   ```

## TODO
- Virtual gamepad/joystick
- Configuration file and argument parsing

## Aknowledgements
- [https://github.com/JibbSmart/GamepadMotionHelpers](https://github.com/JibbSmart/GamepadMotionHelpers)
- [https://github.com/qrasmont/bmi270](https://github.com/qrasmont/bmi270)
