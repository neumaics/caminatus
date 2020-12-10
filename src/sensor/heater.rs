use std::error::Error;

use rppal::gpio::{OutputPin, Gpio};

/// Interface into a zero-crossing solid state relay.
pub struct Heater {
    pin: OutputPin
}

#[derive(Debug)]
pub enum HeaterError {
    GpioError {
        source: rppal::gpio::Error
    }
}

impl From<rppal::gpio::Error> for HeaterError {
    fn from(error: rppal::gpio::Error) -> Self {
        HeaterError::GpioError {
            source: error
        }
    }
}

impl Error for HeaterError {}

impl std::fmt::Display for HeaterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HeaterError::GpioError { source } => write!(f, "Gpio Error {}", source),
        }
    }   
}

impl Heater {
    /// The gpio pin to send the on/off signal. Note, this is the gpio index and
    ///   not the physical gpio pin. That is, GPIO #4 -> Physical pin #7.
    pub fn new(gpio_pin: u8) -> Result<Heater, HeaterError> {
        let pin = Gpio::new()?.get(gpio_pin)?.into_output();

        Ok(Heater {
            pin: pin
        })
    }

    pub fn toggle(&mut self) {
        &self.pin.toggle();
    }

    pub fn on(&mut self) {
        &self.pin.set_high();
    }

    pub fn off(&mut self) {
        &self.pin.set_low();
    }
}
