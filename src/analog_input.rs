use crate::lights::LED;
use ads1x1x::{
    Ads1x1x, channel,
    ic::{Ads1015, Resolution12Bit},
    mode::OneShot,
};
use linux_embedded_hal::I2cdev;
use std::sync::{Arc, Mutex};

pub struct AnalogInput {
    driver: Arc<Mutex<Ads1x1x<I2cdev, Ads1015, Resolution12Bit, OneShot>>>,
    led: Option<LED>,
    channel: u8,
    pub value: f64,
    pub max_value: f64,
}

impl AnalogInput {
    pub fn new(
        driver: Arc<Mutex<Ads1x1x<I2cdev, Ads1015, Resolution12Bit, OneShot>>>,
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

    pub fn read(&mut self) -> Result<f64, String> {
        let value = match self.channel {
            0 => self
                .driver
                .lock()
                .unwrap()
                .read(channel::SingleA0)
                .map_err(|_| "Failed to read value from channel 0".to_string()),
            1 => self
                .driver
                .lock()
                .unwrap()
                .read(channel::SingleA1)
                .map_err(|_| "Failed to read value from channel 1".to_string()),
            2 => self
                .driver
                .lock()
                .unwrap()
                .read(channel::SingleA2)
                .map_err(|_| "Failed to read value from channel 2".to_string()),
            3 => self
                .driver
                .lock()
                .unwrap()
                .read(channel::SingleA3)
                .map_err(|_| "Failed to read value from channel 3".to_string()),
            _ => return Err("Invalid channel".to_string()),
        }?;

        self.value = value as f64 / self.max_value;

        if self.led.is_some() {
            // Update LED brightness based on analog value
            if let Err(e) = self.led.as_mut().unwrap().set_brightness(self.value) {
                return Err(format!("Failed to update LED: {}", e));
            }
        }

        Ok(self.value)
    }
}
