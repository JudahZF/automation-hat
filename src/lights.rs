//! LED control functionality for the Automation HAT.
//!
//! This module provides the `LED` struct, which represents a single LED on the Automation HAT.
//! Each LED has a brightness level that can be controlled from 0.0 to 1.0.

use linux_embedded_hal::I2cdev;
use sn3218_hal::SN3218;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

// Shared global state to track LED brightness values across the system
static LED_STATE: OnceLock<Mutex<HashMap<u8, u8>>> = OnceLock::new();

/// Represents a single LED on the Automation HAT.
///
/// The `LED` struct provides control over a single LED, allowing it to be turned on/off
/// or set to a specific brightness level. LEDs are controlled through the SN3218 LED driver
/// chip which supports 18 channels with 255 brightness levels each.
pub struct LED {
    /// Reference to the SN3218 LED driver
    driver: Arc<Mutex<SN3218<I2cdev>>>,
    /// Channel number on the SN3218 (0-17)
    channel: u8,
    /// Current brightness value (0.0-1.0)
    pub brightness: f64,
    /// Maximum hardware brightness value (typically 255)
    max_brightness: u8,
}

impl LED {
    /// Creates a new LED instance for the specified channel.
    ///
    /// # Arguments
    ///
    /// * `driver` - Shared reference to the SN3218 LED driver
    /// * `channel` - The channel number (0-17) on the SN3218 chip
    ///
    /// # Returns
    ///
    /// A new `LED` instance initialized to off (brightness 0.0)
    pub fn new(driver: Arc<Mutex<SN3218<I2cdev>>>, channel: u8) -> Self {
        // Initialize global LED state if not already done
        LED_STATE.get_or_init(|| Mutex::new(HashMap::new()));

        LED {
            driver,
            channel,
            brightness: 0.0,
            max_brightness: 255,
        }
    }

    /// Turns the LED on at full brightness.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error
    pub fn on(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.set_brightness(1.0)
    }

    /// Turns the LED off (brightness 0.0).
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error
    pub fn off(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.set_brightness(0.0)
    }

    /// Toggles the LED between on and off states.
    ///
    /// If the LED is currently off (brightness 0.0), it will be turned on.
    /// Otherwise, it will be turned off.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error
    pub fn toggle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.brightness == 0.0 {
            self.on()
        } else {
            self.off()
        }
    }

    /// Sets the LED brightness to a specific value.
    ///
    /// # Arguments
    ///
    /// * `brightness` - A value between 0.0 (off) and 1.0 (full brightness)
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error
    ///
    /// # Errors
    ///
    /// Returns an error if the brightness value is outside the valid range of 0.0 to 1.0,
    /// or if communication with the LED driver fails.
    pub fn set_brightness(&mut self, brightness: f64) -> Result<(), Box<dyn std::error::Error>> {
        if brightness < 0.0 || brightness > 1.0 {
            return Err("Brightness must be between 0.0 and 1.0".into());
        }

        self.brightness = brightness;
        let value = (brightness * self.max_brightness as f64) as u8;

        let led_state_mutex = LED_STATE.get_or_init(|| Mutex::new(HashMap::new()));
        let mut led_state = led_state_mutex.lock().unwrap();

        // Update the state for this channel
        led_state.insert(self.channel, value);

        // Prepare values array with current state of all channels
        let mut values = [0u8; 18];
        let mut led_mask = 0u32;

        for (channel, brightness) in led_state.iter() {
            if *channel < 18 {
                values[*channel as usize] = *brightness;
                if *brightness > 0 {
                    led_mask |= 1u32 << channel;
                }
            }
        }

        let mut driver = self.driver.lock().unwrap();
        driver.enable_leds(led_mask).unwrap();
        driver.output(&values).unwrap();

        Ok(())
    }

    /// Alias for `set_brightness` - sets the LED to a specific brightness.
    ///
    /// # Arguments
    ///
    /// * `brightness` - A value between 0.0 (off) and 1.0 (full brightness)
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error
    pub fn set(&mut self, brightness: f64) -> Result<(), Box<dyn std::error::Error>> {
        self.set_brightness(brightness)
    }
}

/// Implement Clone for LED to allow LED objects to be duplicated.
/// This is useful when the same LED needs to be shared between multiple components.
impl Clone for LED {
    fn clone(&self) -> Self {
        LED {
            driver: Arc::clone(&self.driver),
            channel: self.channel,
            brightness: self.brightness,
            max_brightness: self.max_brightness,
        }
    }
}

/// Implement Debug for LED to allow for easier debugging and printing of LED objects.
impl std::fmt::Debug for LED {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LED")
            .field("channel", &self.channel)
            .field("brightness", &self.brightness)
            .field("max_brightness", &self.max_brightness)
            .finish()
    }
}
