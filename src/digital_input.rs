//! Digital input control for Automation HAT boards.
//!
//! This module provides control for the digital input pins on Automation HAT boards.
//! Digital inputs can read 5V signals and have indicator LEDs to show their current state.

use crate::lights::LED;

use embedded_hal::digital::InputPin;
use linux_embedded_hal::{
    CdevPin,
    gpio_cdev::{Line, LineRequestFlags},
};

/// Controls a digital input on the Automation HAT.
///
/// Digital inputs can read 5V signals from external devices. When a 5V signal
/// is detected, the input reads as high (true). Each input can have an associated
/// LED that automatically indicates the input state.
pub struct DigitalInput {
    /// GPIO pin for the digital input
    pin: CdevPin,
    /// Optional LED indicator for this input
    led: Option<LED>,
    /// Whether the LED should automatically reflect input state
    _auto_light: bool,
}

impl DigitalInput {
    /// Creates a new digital input with automatic LED indication enabled.
    ///
    /// # Arguments
    ///
    /// * `line` - GPIO line connected to the digital input
    /// * `led` - Optional LED indicator for this input
    ///
    /// # Returns
    ///
    /// A new `DigitalInput` instance with automatic LED indication enabled
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

    /// Creates a new digital input with configurable LED indication.
    ///
    /// # Arguments
    ///
    /// * `line` - GPIO line connected to the digital input
    /// * `led` - Optional LED indicator for this input
    /// * `auto_light` - Whether the LED should automatically reflect the input state
    ///
    /// # Returns
    ///
    /// A new `DigitalInput` instance with the specified LED behavior
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

    /// Reads the current state of the digital input.
    ///
    /// When auto_light is enabled and an LED is attached, this method will
    /// also update the LED to reflect the current input state.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the input is high (5V signal detected)
    /// * `Ok(false)` - If the input is low (no signal)
    /// * `Err(String)` - If reading the input failed
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
