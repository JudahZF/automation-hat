//! Relay control for Automation HAT boards.
//!
//! This module provides control for the relay outputs on Automation HAT boards.
//! Each relay has both normally open (NO) and normally closed (NC) terminals,
//! and can be controlled with indicator LEDs showing the current state.

use crate::lights::LED;

use embedded_hal::digital::{OutputPin, PinState};
use linux_embedded_hal::{
    CdevPin,
    gpio_cdev::{Line, LineRequestFlags},
};

/// Controls a relay output on the Automation HAT.
///
/// Each relay provides a high-power switch controlled by the Raspberry Pi.
/// Relays have both normally open (NO) and normally closed (NC) terminals,
/// which can be used to switch external circuits.
pub struct Relay {
    /// GPIO pin controlling the relay
    pin: CdevPin,
    /// LED indicating the normally open contact state
    no_led: Option<LED>,
    /// LED indicating the normally closed contact state
    nc_led: Option<LED>,
    /// Whether LEDs should automatically reflect the relay state
    _auto_light: bool,
    /// Current state of the relay (true = activated/on, false = deactivated/off)
    pub value: bool,
}

impl Relay {
    /// Creates a new relay instance with automatic LED indication enabled.
    ///
    /// # Arguments
    ///
    /// * `line` - GPIO line connected to the relay
    /// * `no_led` - Optional LED for the normally open contact indicator
    /// * `nc_led` - Optional LED for the normally closed contact indicator
    ///
    /// # Returns
    ///
    /// A new `Relay` instance configured with automatic LED indication
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

    /// Creates a new relay instance with configurable LED indication.
    ///
    /// # Arguments
    ///
    /// * `line` - GPIO line connected to the relay
    /// * `no_led` - Optional LED for the normally open contact indicator
    /// * `nc_led` - Optional LED for the normally closed contact indicator
    /// * `auto_light` - Whether LEDs should automatically reflect relay state
    ///
    /// # Returns
    ///
    /// A new `Relay` instance with the specified LED behavior
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

    /// Sets the state of the relay.
    ///
    /// When `open` is true, the relay is activated:
    /// - The normally open (NO) contacts close
    /// - The normally closed (NC) contacts open
    /// - If auto_light is enabled, the NO LED lights up and NC LED turns off
    ///
    /// When `open` is false, the relay is deactivated:
    /// - The normally open (NO) contacts open
    /// - The normally closed (NC) contacts close
    /// - If auto_light is enabled, the NO LED turns off and NC LED lights up
    ///
    /// # Arguments
    ///
    /// * `open` - The desired state of the relay (true = activated, false = deactivated)
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error message if the operation failed
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
