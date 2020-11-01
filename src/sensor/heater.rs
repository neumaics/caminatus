use std::error::Error;

use rppal::gpio::{OutputPin, Gpio};

/// Interface into a zero-crossing solid state relay.
pub struct Heater {
    pin: OutputPin
}

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

impl Heater {
    /// The gpio pin to send the on/off signal. Note, this is the gpio index and
    ///   not the physical gpio pin. That is, GPIO #4 -> Physical pin #7.
    pub fn init(gpio_pin: u8) -> Result<Heater, HeaterError> {
        let pin = Gpio::new()?.get(gpio_pin)?.into_output();

        Ok(Heater {
            pin: pin
        })
    }

    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    pub fn turn_on(&mut self) {
        self.pin.set_high();
    }

    pub fn turn_off(&mut self) {
        self.pin.set_low();
    }

    /// Not quite pulse-width-modulation.
    ///   Turn on the heater for the proportion of time given (0-1)
    pub fn proportional(proportion: f32) {
        assert!(proportion > 0.0 && proportion < 1.0);
        
        // Turn on heater for t = proportion, turn off heater otherwise

        // Ensure heater is off
    }
}
