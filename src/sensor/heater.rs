use std::error::Error;

use rppal::gpio::{OutputPin, Gpio};

/// Interface into a zero-crossing solid state relay.
pub struct Heater {
    pin: OutputPin
}

impl Heater {
    /// The gpio pin to send the on/off signal. Note, this is the gpio index and
    ///   not the physical gpio pin. GPIO #4 -> Physical pin #7.
    fn init(gpio_pin: u8) -> Result<Heater, Box<dyn Error>> {
        let mut pin = Gpio::new()?.get(gpio_pin)?.into_output();

        Ok(Heater {
            pin: pin
        })
    }
}
