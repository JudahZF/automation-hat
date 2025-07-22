use crate::lights::LED;

use embedded_hal::digital::{OutputPin, PinState};
use linux_embedded_hal::{
    CdevPin,
    gpio_cdev::{Line, LineRequestFlags},
};

pub struct Relay {
    pin: CdevPin,
    no_led: Option<LED>,
    nc_led: Option<LED>,
    _auto_light: bool,
    pub value: bool,
}

impl Relay {
    pub fn new(line: Line, no_led: Option<LED>, nc_led: Option<LED>) -> Self {
        let line = line
            .request(LineRequestFlags::OUTPUT, 0, "AutomationHAT Rust SDK")
            .unwrap();
        let pin = CdevPin::new(line).unwrap();
        Relay {
            pin,
            no_led,
            nc_led,
            _auto_light: true,
            value: false,
        }
    }

    pub fn new_with_auto_light(
        line: Line,
        no_led: Option<LED>,
        nc_led: Option<LED>,
        auto_light: bool,
    ) -> Self {
        let line = line
            .request(LineRequestFlags::OUTPUT, 0, "AutomationHAT Rust SDK")
            .unwrap();
        let pin = CdevPin::new(line).unwrap();
        Relay {
            pin,
            no_led,
            nc_led,
            _auto_light: auto_light,
            value: false,
        }
    }

    pub fn write(&mut self, open: bool) -> Result<(), &str> {
        if self._auto_light {
            let no_brightness = match open {
                true => 1.0,
                false => 0.0,
            };
            let nc_brightness = match open {
                true => 0.0,
                false => 1.0,
            };
            if self.no_led.is_some() {
                let _ = self.no_led.as_mut().unwrap().set(no_brightness);
            }
            if self.nc_led.is_some() {
                let _ = self.nc_led.as_mut().unwrap().set(nc_brightness);
            }
        }
        match self.pin.set_state(match open {
            true => PinState::High,
            false => PinState::Low,
        }) {
            Ok(_) => {}
            Err(_) => return Err("Unable to set value"),
        };
        self.value = open;
        Ok(())
    }
}
