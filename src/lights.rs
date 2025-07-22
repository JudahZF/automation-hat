use linux_embedded_hal::I2cdev;
use sn3218_hal::SN3218;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

static LED_STATE: OnceLock<Mutex<HashMap<u8, u8>>> = OnceLock::new();

pub struct LED {
    driver: Arc<Mutex<SN3218<I2cdev>>>,
    channel: u8,
    pub brightness: f64,
    max_brightness: u8,
}

impl LED {
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

    pub fn on(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.set_brightness(1.0)
    }

    pub fn off(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.set_brightness(0.0)
    }

    pub fn toggle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.brightness == 0.0 {
            self.on()
        } else {
            self.off()
        }
    }

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

    pub fn set(&mut self, brightness: f64) -> Result<(), Box<dyn std::error::Error>> {
        self.set_brightness(brightness)
    }
}

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

impl std::fmt::Debug for LED {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LED")
            .field("channel", &self.channel)
            .field("brightness", &self.brightness)
            .field("max_brightness", &self.max_brightness)
            .finish()
    }
}
