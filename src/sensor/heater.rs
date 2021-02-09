use std::error::Error;

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

#[cfg(target = "armv7-unknown-linux-gnueabihf")]
pub mod real {
    use super::*;
    use rppal::gpio::{OutputPin, Gpio};

    /// Interface into a zero-crossing solid state relay.
    pub struct Heater {
        pin: OutputPin
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
}

#[cfg(not(target = "armv7-unknown-linux-gnueabihf"))]
pub mod simulated {
    use super::*;

    pub struct Heater {
        pin: u8
    }

    impl Heater {
        /// The gpio pin to send the on/off signal. Note, this is the gpio index and
        ///   not the physical gpio pin. That is, GPIO #4 -> Physical pin #7.
        pub fn new(gpio_pin: u8) -> Result<Heater, HeaterError> {
            Ok(Heater {
                pin: gpio_pin
            })
        }

        pub fn toggle(&mut self) {
            // &self.pin.toggle();
        }

        pub fn on(&mut self) {
            // &self.pin.set_high();
        }

        pub fn off(&mut self) {
            // &self.pin.set_low();
        }
    }
}
