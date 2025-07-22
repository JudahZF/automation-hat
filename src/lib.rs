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

use ads1x1x::{Ads1x1x, TargetAddr};
use linux_embedded_hal::{
    CdevPin, I2cdev, SpidevDevice,
    gpio_cdev::{Chip, LineRequestFlags},
};
use sn3218_hal::SN3218;
use st7735_lcd::ST7735;
use std::sync::{Arc, Mutex};

static RELAY_1: u32 = 13;
static RELAY_2: u32 = 19;
static RELAY_3: u32 = 16;

static INPUT_1: u32 = 26;
static INPUT_2: u32 = 20;
static INPUT_3: u32 = 21;

static OUTPUT_1: u32 = 5;
static OUTPUT_2: u32 = 12;
static OUTPUT_3: u32 = 6;

pub enum HatType {
    AutomationHAT,
    AutomationPHAT,
    AutomationHATMini,
}

pub struct Relays {
    pub one: Option<Relay>,
    pub two: Option<Relay>,
    pub three: Relay,
}

impl Relays {
    pub fn new(one: Option<Relay>, two: Option<Relay>, three: Relay) -> Self {
        Relays { one, two, three }
    }
}

pub struct Inputs {
    pub one: DigitalInput,
    pub two: DigitalInput,
    pub three: DigitalInput,
}

impl Inputs {
    pub fn new(one: DigitalInput, two: DigitalInput, three: DigitalInput) -> Self {
        Inputs { one, two, three }
    }
}

pub struct Outputs {
    pub one: DigitalOutput,
    pub two: DigitalOutput,
    pub three: DigitalOutput,
}

impl Outputs {
    pub fn new(one: DigitalOutput, two: DigitalOutput, three: DigitalOutput) -> Self {
        Outputs { one, two, three }
    }
}

pub struct AnalogInputs {
    pub one: AnalogInput,
    pub two: AnalogInput,
    pub three: AnalogInput,
}

impl AnalogInputs {
    pub fn new(one: AnalogInput, two: AnalogInput, three: AnalogInput) -> Self {
        AnalogInputs { one, two, three }
    }
}

pub struct AutomationHAT {
    pub hat_type: HatType,
    pub relays: Relays,
    pub inputs: Inputs,
    pub outputs: Outputs,
    pub analog_inputs: AnalogInputs,
    pub display: Option<ST7735<SpidevDevice, CdevPin, CdevPin>>,
}

impl AutomationHAT {
    pub fn new(hat_type: HatType) -> Self {
        let i2c_analog = I2cdev::new("/dev/i2c-1").unwrap();
        let analog_driver = Arc::new(Mutex::new(Ads1x1x::new_ads1015(
            i2c_analog,
            TargetAddr::default(),
        )));

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
