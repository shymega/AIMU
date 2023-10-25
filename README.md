# AIMU

- Userspace IMU-assisted aiming for Linux.
- Maps accelerometer+gyroscope motions to virtual mouse movements.
- Accounts for angle between screen and keyboard (configurable).

## Usage
1. Prepare the system.
   1. *BMI160 only:* Disable `bmi160_i2c` and `bmi160_core` kernel modules.
      ```shell
      sudo rmmod bmi160_i2c bmi160_core
      ```
   1. Enable `i2c_dev` kernel module.
      ```shell
      sudo modprobe i2c_dev
      ```
   1. Add user to `i2c` group.
      ```shell
      sudo usermod -aG i2c $(whoami)
      newgrp i2c
      ```
1. Build and run:
   1. *Default:* Dynamic dispatch (less performant, more convenient):
      1. Build and run:
         ```shell
         cargo run --release -- --help
         ```
   1. Static dispatch (more performant, less convenient):
      1. Tweak default values in source code.
      1. Build and run:
         ```shell
         # Optional: add `--features cli` for command line options
         cargo run --release --features bmi160 -- --help
         # or
         cargo run --release --features bmi260 -- --help
         ```

## TODO
- Revisit virtual gamepad/joystick (separate branch)
- Expand trigger mappings
- Add configuration file and env parsing

## Aknowledgements
- [https://github.com/JibbSmart/GamepadMotionHelpers](https://github.com/JibbSmart/GamepadMotionHelpers)
- [https://github.com/qrasmont/bmi270](https://github.com/qrasmont/bmi270)
