use crate::lights::LED;

use embedded_hal::digital::InputPin;
use linux_embedded_hal::{
    CdevPin,
    gpio_cdev::{Line, LineRequestFlags},
};

pub struct DigitalInput {
    pin: CdevPin,
    led: Option<LED>,
    _auto_light: bool,
}

impl DigitalInput {
    pub fn new(line: Line, led: Option<LED>) -> Self {
        let line = line
            .request(LineRequestFlags::INPUT, 0, "AutomationHAT Rust SDK")
            .unwrap();
        let pin = CdevPin::new(line).unwrap();
        DigitalInput {
            pin,
            led,
            _auto_light: true,
        }
    }

    pub fn new_with_auto_light(line: Line, led: Option<LED>, auto_light: bool) -> Self {
        let line = line
            .request(LineRequestFlags::INPUT, 0, "AutomationHAT Rust SDK")
            .unwrap();
        let pin = CdevPin::new(line).unwrap();
        DigitalInput {
            pin,
            led,
            _auto_light: auto_light,
        }
    }

    pub fn read(&mut self) -> Result<bool, String> {
        let value = self.pin.is_high().map_err(|e| e.to_string())?;
        if self._auto_light && self.led.is_some() {
            if let Err(e) = self.led.as_mut().unwrap().set_brightness(match value {
                true => 1.0,
                false => 0.0,
            }) {
                println!("Failed to update LED: {}", e);
            }
        }
        Ok(value)
    }
}
