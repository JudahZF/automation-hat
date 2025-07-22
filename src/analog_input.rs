//! Analog input control for Automation HAT boards.
//!
//! This module provides control for the analog input pins on Automation HAT boards.
//! Analog inputs can read variable voltage levels and have indicator LEDs
//! that can show input levels proportionally.

use crate::lights::LED;
use ads1x1x::{
    Ads1x1x, channel,
    ic::{Ads1015, Resolution12Bit},
    mode::OneShot,
};
use linux_embedded_hal::I2cdev;
use std::sync::{Arc, Mutex};

/// Controls an analog input on the Automation HAT.
///
/// Analog inputs can read variable voltage levels from external devices.
/// The input values are read through an ADS1015 analog-to-digital converter
/// and normalized to a value between 0.0 and 1.0 based on the max_value.
/// Each input can have an associated LED that indicates the input level.
pub struct AnalogInput {
    /// Reference to the ADS1015 ADC driver
    driver: Arc<Mutex<Ads1x1x<I2cdev, Ads1015, Resolution12Bit, Continuous>>>,
    /// Optional LED indicator for this input
    led: Option<LED>,
    /// Channel number on the ADS1015 (0-3)
    channel: u8,
    /// Current normalized value (0.0-1.0)
    pub value: f64,
    /// Maximum raw ADC value used for normalization
    pub max_value: f64,
}

impl AnalogInput {
    /// Creates a new analog input for the specified ADC channel.
    ///
    /// # Arguments
    ///
    /// * `driver` - Shared reference to the ADS1015 ADC driver
    /// * `led` - Optional LED indicator for this input
    /// * `channel` - The channel number (0-3) on the ADS1015 ADC
    ///
    /// # Returns
    ///
    /// A new `AnalogInput` instance with the specified channel and LED
    pub fn new(
        driver: Arc<Mutex<Ads1x1x<I2cdev, Ads1015, Resolution12Bit, Continuous>>>,
        led: Option<LED>,
        channel: u8,
    ) -> Self {
        AnalogInput {
            driver,
            led,
            channel,
            value: 0.0,
            max_value: 25.85,
        }
    }

    /// Reads the current value from the analog input.
    ///
    /// This method reads a raw value from the ADC, normalizes it to a value between
    /// 0.0 and 1.0 based on the max_value, and updates the LED brightness to match
    /// if an LED is attached.
    ///
    /// # Returns
    ///
    /// * `Ok(f64)` - The normalized input value between 0.0 and 1.0
    /// * `Err(String)` - If reading the input or updating the LED failed
    pub fn read(&mut self) -> Result<f64, String> {
        let mut driver = self.driver.lock().unwrap();
        match self.channel {
            0 => driver
                .select_channel(channel::SingleA0)
                .map_err(|error| format!("Failed to read value from channel 0: {:?}", error)),
            1 => driver
                .select_channel(channel::SingleA1)
                .map_err(|error| format!("Failed to read value from channel 1: {:?}", error)),
            2 => driver
                .select_channel(channel::SingleA2)
                .map_err(|error| format!("Failed to read value from channel 2: {:?}", error)),
            3 => driver
                .select_channel(channel::SingleA3)
                .map_err(|error| format!("Failed to read value from channel 3: {:?}", error)),
            _ => return Err("Invalid channel".to_string()),
        }?;

        let value = driver.read().unwrap();

        self.value = ((value as f64 / 10.0) * 2.048) / self.max_value;

        if self.led.is_some() {
            // Update LED brightness based on analog value
            if let Err(e) = self.led.as_mut().unwrap().set_brightness(self.value) {
                return Err(format!("Failed to update LED: {}", e));
            }
        }

        Ok(self.value)
    }
}
