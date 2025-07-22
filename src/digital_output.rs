use crate::lights::LED;

use embedded_hal::digital::{OutputPin, PinState};
use linux_embedded_hal::{
    CdevPin,
    gpio_cdev::{Line, LineRequestFlags},
};

pub struct DigitalOutput {
    pin: CdevPin,
    led: Option<LED>,
    _auto_light: bool,
    pub value: bool,
}

impl DigitalOutput {
    pub fn new(line: Line, led: Option<LED>) -> Self {
        let line = line
            .request(LineRequestFlags::OUTPUT, 0, "AutomationHAT Rust SDK")
            .unwrap();
        let pin = CdevPin::new(line).unwrap();
        DigitalOutput {
            pin,
            led,
            _auto_light: true,
            value: false,
        }
    }

    pub fn new_with_auto_light(line: Line, led: Option<LED>, auto_light: bool) -> Self {
        let line = line
            .request(LineRequestFlags::OUTPUT, 0, "AutomationHAT Rust SDK")
            .unwrap();
        let pin = CdevPin::new(line).unwrap();
        DigitalOutput {
            pin,
            led,
            _auto_light: auto_light,
            value: false,
        }
    }

    pub fn write(&mut self, on: bool) -> Result<(), String> {
        if self._auto_light {
            if let Some(led) = &mut self.led {
                match led.set(match on {
                    true => 1.0,
                    false => 0.0,
                }) {
                    Ok(_) => {}
                    Err(e) => return Err(format!("Unable to set LED state: {}", e)),
                }
            }
        }
        return match self.pin.set_state(match on {
            true => PinState::High,
            false => PinState::Low,
        }) {
            Ok(_) => {
                self.value = on;
                Ok(())
            }
            Err(e) => Err(format!("Unable to set pin state: {}", e)),
        };
    }
}
