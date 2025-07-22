//! Digital output control for Automation HAT boards.
//!
//! This module provides control for the digital output pins on Automation HAT boards.
//! Digital outputs provide 5V signals for controlling external devices and have indicator
//! LEDs to show their current state.

use crate::lights::LED;

use embedded_hal::digital::{OutputPin, PinState};
use linux_embedded_hal::{
    CdevPin,
    gpio_cdev::{Line, LineRequestFlags},
};

/// Controls a digital output on the Automation HAT.
///
/// Digital outputs provide 5V signals for controlling external devices.
/// When an output is set high, it outputs 5V. Each output can have an associated
/// LED that automatically indicates the output state.
pub struct DigitalOutput {
    /// GPIO pin for the digital output
    pin: CdevPin,
    /// Optional LED indicator for this output
    led: Option<LED>,
    /// Whether the LED should automatically reflect output state
    _auto_light: bool,
    /// Current state of the output (true = high/on, false = low/off)
    pub value: bool,
}

impl DigitalOutput {
    /// Creates a new digital output with automatic LED indication enabled.
    ///
    /// # Arguments
    ///
    /// * `line` - GPIO line connected to the digital output
    /// * `led` - Optional LED indicator for this output
    ///
    /// # Returns
    ///
    /// A new `DigitalOutput` instance with automatic LED indication enabled
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

    /// Creates a new digital output with configurable LED indication.
    ///
    /// # Arguments
    ///
    /// * `line` - GPIO line connected to the digital output
    /// * `led` - Optional LED indicator for this output
    /// * `auto_light` - Whether the LED should automatically reflect the output state
    ///
    /// # Returns
    ///
    /// A new `DigitalOutput` instance with the specified LED behavior
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

    /// Sets the state of the digital output.
    ///
    /// When `on` is true, the output is set high (5V).
    /// When `on` is false, the output is set low (0V).
    /// If auto_light is enabled and an LED is attached, this method will
    /// also update the LED to reflect the current output state.
    ///
    /// # Arguments
    ///
    /// * `on` - The desired state of the output (true = high/on, false = low/off)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the output was successfully set
    /// * `Err(String)` - If setting the output or LED failed, with an error message
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
