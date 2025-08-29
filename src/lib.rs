//! # Automation HAT Rust Library
//!
//! A Rust library for controlling the [Pimoroni Automation HAT](https://shop.pimoroni.com/products/automation-hat),
//! [Automation pHAT](https://shop.pimoroni.com/products/automation-phat), and
//! [Automation HAT Mini](https://shop.pimoroni.com/products/automation-hat-mini).
//!
//! This library provides a convenient interface to control all features of the Automation HAT devices,
//! including relays, digital outputs, digital inputs, analog inputs, and LEDs. The library also supports
//! the display on the Automation HAT Mini.
//!
//! ## Features
//!
//! - Control relays, digital outputs, and read digital/analog inputs
//! - Full LED control with automatic status indication
//! - Support for all Automation HAT variants (HAT, pHAT, Mini)
//! - Display support for Automation HAT Mini
//!
//! ## Example
//!
//! ```rust,no_run
//! use automation_hat::{AutomationHAT, HatType};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new AutomationHAT instance
//!     let mut hat = AutomationHAT::new(HatType::AutomationHAT);
//!
//!     // Toggle relay 3
//!     hat.relays.three.write(true)?;
//!
//!     // Read digital input 1
//!     let input_value = hat.inputs.one.read()?;
//!     println!("Input 1: {}", input_value);
//!
//!     // Set digital output 2
//!     hat.outputs.two.write(true)?;
//!
//!     // Read analog input 3
//!     let analog_value = hat.analog_inputs.three.read()?;
//!     println!("Analog 3: {}", analog_value);
//!
//!     Ok(())
//! }
//! ```

mod analog_input;
mod digital_input;
mod digital_output;
mod lights;
mod relay;

pub use analog_input::AnalogInput;
pub use digital_input::DigitalInput;
pub use digital_output::DigitalOutput;
pub use lights::LED;
pub use relay::Relay;

use ads1x1x::{Ads1x1x, FullScaleRange, TargetAddr};
use linux_embedded_hal::{
    CdevPin, I2cdev, SpidevDevice,
    gpio_cdev::{Chip, LineRequestFlags},
};
use sn3218_hal::SN3218;
use st7735_lcd::ST7735;
use std::sync::{Arc, Mutex};

static RELAY_3: u32 = 13;
static RELAY_2: u32 = 19;
static RELAY_1: u32 = 16;

static INPUT_1: u32 = 26;
static INPUT_2: u32 = 20;
static INPUT_3: u32 = 21;

static OUTPUT_1: u32 = 5;
static OUTPUT_2: u32 = 12;
static OUTPUT_3: u32 = 6;

/// Represents the type of Automation HAT hardware being used.
///
/// Different HAT types have different capabilities:
/// - `AutomationHAT`: Full-size HAT with 3 relays, LEDs for all I/O
/// - `AutomationPHAT`: Smaller pHAT form factor with fewer features
/// - `AutomationHATMini`: Mini form factor with LCD display
pub enum HatType {
    /// Full-sized Automation HAT with 3 relays and status LEDs for all I/O
    AutomationHAT,
    /// Smaller pHAT form factor with fewer features than the full HAT
    AutomationPHAT,
    /// Compact form factor with 0.96" color LCD display
    AutomationHATMini,
}

/// Container for relay controls on the Automation HAT.
///
/// Provides access to the relays on the Automation HAT:
/// - `one` and `two` are optional as they're not present on all HAT variants
/// - `three` is available on all HAT variants
pub struct Relays {
    /// Relay 1 - Only present on full HAT
    pub one: Option<Relay>,
    /// Relay 2 - Only present on full HAT
    pub two: Option<Relay>,
    /// Relay 3 - Present on all HAT variants
    pub three: Relay,
}

impl Relays {
    /// Creates a new Relays container with the specified relay instances.
    ///
    /// # Arguments
    ///
    /// * `one` - Optional Relay 1 instance (present on HAT)
    /// * `two` - Optional Relay 2 instance (present on HAT)
    /// * `three` - Relay 3 instance (present on all variants)
    pub fn new(one: Option<Relay>, two: Option<Relay>, three: Relay) -> Self {
        Relays { one, two, three }
    }
}

/// Container for digital input controls on the Automation HAT.
///
/// Provides access to the three digital inputs available on all HAT variants.
pub struct Inputs {
    /// Digital Input 1
    pub one: DigitalInput,
    /// Digital Input 2
    pub two: DigitalInput,
    /// Digital Input 3
    pub three: DigitalInput,
}

impl Inputs {
    /// Creates a new Inputs container with the specified digital input instances.
    ///
    /// # Arguments
    ///
    /// * `one` - Digital Input 1 instance
    /// * `two` - Digital Input 2 instance
    /// * `three` - Digital Input 3 instance
    pub fn new(one: DigitalInput, two: DigitalInput, three: DigitalInput) -> Self {
        Inputs { one, two, three }
    }
}

/// Container for digital output controls on the Automation HAT.
///
/// Provides access to the three digital outputs available on all HAT variants.
pub struct Outputs {
    /// Digital Output 1
    pub one: DigitalOutput,
    /// Digital Output 2
    pub two: DigitalOutput,
    /// Digital Output 3
    pub three: DigitalOutput,
}

impl Outputs {
    /// Creates a new Outputs container with the specified digital output instances.
    ///
    /// # Arguments
    ///
    /// * `one` - Digital Output 1 instance
    /// * `two` - Digital Output 2 instance
    /// * `three` - Digital Output 3 instance
    pub fn new(one: DigitalOutput, two: DigitalOutput, three: DigitalOutput) -> Self {
        Outputs { one, two, three }
    }
}

/// Container for analog input controls on the Automation HAT.
///
/// Provides access to the three analog inputs available on all HAT variants.
pub struct AnalogInputs {
    /// Analog Input 1
    pub one: AnalogInput,
    /// Analog Input 2
    pub two: AnalogInput,
    /// Analog Input 3
    pub three: AnalogInput,
}

impl AnalogInputs {
    /// Creates a new AnalogInputs container with the specified analog input instances.
    ///
    /// # Arguments
    ///
    /// * `one` - Analog Input 1 instance
    /// * `two` - Analog Input 2 instance
    /// * `three` - Analog Input 3 instance
    pub fn new(one: AnalogInput, two: AnalogInput, three: AnalogInput) -> Self {
        AnalogInputs { one, two, three }
    }
}

/// Main interface for the Automation HAT family of boards.
///
/// This struct provides access to all features of the Automation HAT:
/// - Relays for high-power switching
/// - Digital inputs for reading 5V signals
/// - Digital outputs for 5V control signals
/// - Analog inputs for reading variable voltage levels
/// - Optional display (only on Automation HAT Mini)
pub struct AutomationHAT {
    /// The type of Automation HAT hardware being used
    pub hat_type: HatType,
    /// Access to relay controls
    pub relays: Relays,
    /// Access to digital input controls
    pub inputs: Inputs,
    /// Access to digital output controls
    pub outputs: Outputs,
    /// Access to analog input controls
    pub analog_inputs: AnalogInputs,
    /// Access to the ST7735 display (only available on Automation HAT Mini)
    pub display: Option<ST7735<SpidevDevice, CdevPin, CdevPin>>,
}

impl AutomationHAT {
    /// Creates a new AutomationHAT instance for the specified HAT type.
    ///
    /// This initializes all hardware connections, GPIO pins, I2C devices, and
    /// the display (if using the Automation HAT Mini).
    ///
    /// # Arguments
    ///
    /// * `hat_type` - The type of Automation HAT to initialize
    ///
    /// # Returns
    ///
    /// A fully configured `AutomationHAT` instance ready for use
    ///
    /// # Examples
    ///
    /// ```
    /// use automation_hat::{AutomationHAT, HatType};
    ///
    /// // Create a new AutomationHAT instance
    /// let mut hat = AutomationHAT::new(HatType::AutomationHAT);
    /// ```
    pub fn new(hat_type: HatType) -> Self {
        let i2c_analog = I2cdev::new("/dev/i2c-1").unwrap();
        let mut analog_driver = Ads1x1x::new_ads1015(i2c_analog, TargetAddr::default());

        analog_driver
            .set_full_scale_range(FullScaleRange::Within2_048V)
            .unwrap();

        let analog_driver = match analog_driver.into_continuous() {
            Ok(driver) => Arc::new(Mutex::new(driver)),
            Err(_) => panic!("Failed to convert analog driver into continuous mode"),
        };

        let mut gpio_chip = Chip::new("/dev/gpiochip0").unwrap();

        // For AutomationHATMini, disable auto-lighting since there are no LEDs
        let auto_light = !matches!(hat_type, HatType::AutomationHATMini);

        let mut relay_1 = None;
        let mut relay_2 = None;

        let mut relay_3_no_led = None;
        let mut relay_3_nc_led = None;
        let mut input_1_led = None;
        let mut input_2_led = None;
        let mut input_3_led = None;
        let mut output_1_led = None;
        let mut output_2_led = None;
        let mut output_3_led = None;
        let mut analog_input_1_led = None;
        let mut analog_input_2_led = None;
        let mut analog_input_3_led = None;
        let mut display = None;

        match hat_type {
            HatType::AutomationHAT => {
                let i2c_led = I2cdev::new("/dev/i2c-1").unwrap();
                let driver = Arc::new(Mutex::new(SN3218::new(i2c_led)));

                analog_input_1_led = Some(LED::new(driver.clone(), 0));
                analog_input_2_led = Some(LED::new(driver.clone(), 1));
                analog_input_3_led = Some(LED::new(driver.clone(), 2));

                output_1_led = Some(LED::new(driver.clone(), 3));
                output_2_led = Some(LED::new(driver.clone(), 4));
                output_3_led = Some(LED::new(driver.clone(), 5));

                input_1_led = Some(LED::new(driver.clone(), 14));
                input_2_led = Some(LED::new(driver.clone(), 13));
                input_3_led = Some(LED::new(driver.clone(), 12));

                let relay_1_no_led = Some(LED::new(driver.clone(), 6));
                let relay_1_nc_led = Some(LED::new(driver.clone(), 7));
                let relay_2_no_led = Some(LED::new(driver.clone(), 8));
                let relay_2_nc_led = Some(LED::new(driver.clone(), 9));
                relay_3_no_led = Some(LED::new(driver.clone(), 10));
                relay_3_nc_led = Some(LED::new(driver.clone(), 11));

                relay_1 = Some(Relay::new_with_auto_light(
                    gpio_chip.get_line(RELAY_1).unwrap(),
                    relay_1_no_led,
                    relay_1_nc_led,
                    auto_light,
                ));

                relay_2 = Some(Relay::new_with_auto_light(
                    gpio_chip.get_line(RELAY_2).unwrap(),
                    relay_2_no_led,
                    relay_2_nc_led,
                    auto_light,
                ));
            }
            HatType::AutomationPHAT => {}
            HatType::AutomationHATMini => {
                let dc = gpio_chip.get_line(9).unwrap();
                let dc = dc
                    .request(LineRequestFlags::OUTPUT, 0, "AutomationHAT Rust SDK")
                    .unwrap();
                let dc = CdevPin::new(dc).unwrap();

                let rst = gpio_chip.get_line(22).unwrap();
                let rst = rst
                    .request(LineRequestFlags::OUTPUT, 0, "AutomationHAT Rust SDK")
                    .unwrap();
                let rst = CdevPin::new(rst).unwrap();
                display = Some(ST7735::new(
                    SpidevDevice::open("/dev/spidev0.1").unwrap(),
                    dc,
                    rst,
                    false,
                    true,
                    80,
                    160,
                ));

                if let Some(ref mut disp) = display {
                    let mut delay = linux_embedded_hal::Delay {};
                    disp.init(&mut delay).unwrap();
                    disp.set_offset(26, 2);
                }
            }
        }

        let relay_3 = Relay::new_with_auto_light(
            gpio_chip.get_line(RELAY_3).unwrap(),
            relay_3_no_led,
            relay_3_nc_led,
            auto_light,
        );

        let input_1 = DigitalInput::new_with_auto_light(
            gpio_chip.get_line(INPUT_1).unwrap(),
            input_1_led,
            auto_light,
        );
        let input_2 = DigitalInput::new_with_auto_light(
            gpio_chip.get_line(INPUT_2).unwrap(),
            input_2_led,
            auto_light,
        );
        let input_3 = DigitalInput::new_with_auto_light(
            gpio_chip.get_line(INPUT_3).unwrap(),
            input_3_led,
            auto_light,
        );
        let output_1 = DigitalOutput::new_with_auto_light(
            gpio_chip.get_line(OUTPUT_1).unwrap(),
            output_1_led,
            auto_light,
        );
        let output_2 = DigitalOutput::new_with_auto_light(
            gpio_chip.get_line(OUTPUT_2).unwrap(),
            output_2_led,
            auto_light,
        );
        let output_3 = DigitalOutput::new_with_auto_light(
            gpio_chip.get_line(OUTPUT_3).unwrap(),
            output_3_led,
            auto_light,
        );
        let analog_input_1 = AnalogInput::new(analog_driver.clone(), analog_input_1_led, 0);
        let analog_input_2 = AnalogInput::new(analog_driver.clone(), analog_input_2_led, 1);
        let analog_input_3 = AnalogInput::new(analog_driver.clone(), analog_input_3_led, 2);

        let analog_inputs = AnalogInputs::new(analog_input_1, analog_input_2, analog_input_3);
        let inputs = Inputs::new(input_1, input_2, input_3);
        let outputs = Outputs::new(output_1, output_2, output_3);
        let relays = Relays::new(relay_1, relay_2, relay_3);

        Self {
            analog_inputs,
            display,
            hat_type,
            inputs,
            outputs,
            relays,
        }
    }
}
