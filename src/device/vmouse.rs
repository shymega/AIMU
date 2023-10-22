use anyhow::Result;
use evdev::{
    self,
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, EventType, InputEvent, RelativeAxisType,
};

pub struct VMouse {
    device: VirtualDevice,
}

impl VMouse {
    pub fn new() -> Result<Self> {
        Ok(Self {
            device: VirtualDeviceBuilder::new()?
                .name("AIMU")
                .with_relative_axes(&AttributeSet::from_iter([
                    RelativeAxisType::REL_X,
                    RelativeAxisType::REL_Y,
                    RelativeAxisType::REL_WHEEL, // convinces libinput it's a mouse
                ]))?
                .build()?,
        })
    }

    pub fn update(&mut self, x: i32, y: i32) -> Result<()> {
        Ok(self.device.emit(&[
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_X.0, x),
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_Y.0, y),
        ])?)
    }
}
