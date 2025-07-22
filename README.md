# Automation HAT Rust Library

A Rust library for controlling the [Pimoroni Automation HAT](https://shop.pimoroni.com/products/automation-hat), [Automation pHAT](https://shop.pimoroni.com/products/automation-phat), and [Automation HAT Mini](https://shop.pimoroni.com/products/automation-hat-mini).

## Features

- Control relays, digital outputs, and read digital/analog inputs
- Full LED control with automatic status indication
- Support for all Automation HAT variants (HAT, pHAT, Mini)
- Display support for Automation HAT Mini

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
automation-hat = "0.1.0"
```

## Hardware Setup

This library requires I2C and SPI to be enabled on your Raspberry Pi. You can enable these interfaces using `raspi-config`.

## Usage Examples

### Basic Usage

```rust
use automation_hat::{AutomationHAT, HatType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new AutomationHAT instance
    let mut hat = AutomationHAT::new(HatType::AutomationHAT);
    
    // Toggle relay 3
    hat.relays.three.write(true)?;
    
    // Read digital input 1
    let input_value = hat.inputs.one.read()?;
    println!("Input 1: {}", input_value);
    
    // Set digital output 2
    hat.outputs.two.write(true)?;
    
    // Read analog input 3
    let analog_value = hat.analog_inputs.three.read()?;
    println!("Analog 3: {}", analog_value);
    
    Ok(())
}
```

### Automation HAT Mini with Display

```rust
use automation_hat::{AutomationHAT, HatType};
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyleBuilder},
    text::{Baseline, Text},
};
use embedded_graphics_core::primitives::Rectangle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut hat = AutomationHAT::new(HatType::AutomationHATMini);
    
    if let Some(ref mut display) = hat.display {
        // Clear display
        let style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLACK)
            .build();
        
        Rectangle::new(Point::new(0, 0), Size::new(160, 80))
            .into_styled(style)
            .draw(display)?;
            
        // Draw some text
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(Rgb565::WHITE)
            .build();
            
        Text::with_baseline("Automation HAT", Point::new(10, 20), text_style, Baseline::Top)
            .draw(display)?;
    }
    
    Ok(())
}
```

## Device Support

The library supports:

- **Automation HAT**: 3 relays, 3 analog inputs, 3 digital inputs, 3 digital outputs, with indicator LEDs for all channels
- **Automation pHAT**: 1 relay, 3 analog inputs, 3 digital inputs, 3 digital outputs, no indicator LEDs
- **Automation HAT Mini**: 3 relays, 3 analog inputs, 3 digital inputs, 3 digital outputs, no indicator LEDs, but includes a 0.96" color LCD display

## API Reference

### Initialization

```rust
// Create an AutomationHAT instance
let mut hat = AutomationHAT::new(HatType::AutomationHAT);
```

You can choose between:
- `HatType::AutomationHAT`
- `HatType::AutomationPHAT`
- `HatType::AutomationHATMini`

### Relays

Relays provide a high-power switch controlled by the Raspberry Pi.

```rust
// Set relay state (true = on, false = off)
hat.relays.one.write(true)?; // Only available on HAT and pHAT
hat.relays.two.write(true)?; // Only available on HAT and pHAT
hat.relays.three.write(true)?;

// Get relay state
let state = hat.relays.three.value;
```

### Digital Outputs

Digital outputs provide 5V signals for controlling external devices.

```rust
// Set output state (true = on, false = off)
hat.outputs.one.write(true)?;
hat.outputs.two.write(true)?;
hat.outputs.three.write(true)?;

// Get output state
let state = hat.outputs.one.value;
```

### Digital Inputs

Digital inputs read 5V signals from external devices.

```rust
// Read input state (returns true for high, false for low)
let input1 = hat.inputs.one.read()?;
let input2 = hat.inputs.two.read()?;
let input3 = hat.inputs.three.read()?;
```

### Analog Inputs

Analog inputs read variable voltage levels from external devices.

```rust
// Read analog value (returns a float from 0.0 to 1.0)
let analog1 = hat.analog_inputs.one.read()?;
let analog2 = hat.analog_inputs.two.read()?;
let analog3 = hat.analog_inputs.three.read()?;
```

### Display (Automation HAT Mini only)

The Automation HAT Mini includes a 0.96" 160x80 color LCD display.

```rust
if let Some(ref mut display) = hat.display {
    // Use embedded-graphics to draw to the display
    // See embedded-graphics documentation for more details
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgements

- [Pimoroni](https://pimoroni.com/) for creating the Automation HAT hardware
- [embedded-hal](https://github.com/rust-embedded/embedded-hal) for the Rust embedded abstractions