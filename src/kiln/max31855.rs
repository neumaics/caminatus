// Expected to use MAX31855 Thermocouple to digital converter.
//   See: https://datasheets.maximintegrated.com/en/ds/MAX31855.pdf

use bitbang_hal::spi::MODE_0;
use bitbang_hal::spi::SPI;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use crate::kiln::thermocouple::{Thermocouple, ThermocoupleError};

const CLOCK_FREQUENCY: u32 = 1_000_000;

// Only available on raspberry pi compatible platforms
pub struct Max_31855 {
    initialized: bool,
}
impl Thermocouple for Max_31855 {
    fn new(clock_pin: u8, cs_pin: u8, miso: u8) -> Self {
        // let mut spi = SPI::new(MODE_0, miso, mosi, sck, tmr);

        Max_31855 {
            initialized: false,
            // spi: spi,
        }
    }

    fn begin(mut self) -> bool {
        self.initialized = true;
        true
    }

    fn read_internal(self) -> f64 {
        0.0
    }

    fn read(self) -> f64 {
        0.0
    }

    fn read_error(self) -> ThermocoupleError {
        // TODO: Read from 
        ThermocoupleError::Fault
    }
}

//
// https://www.raspberrypi.org/documentation/hardware/raspberrypi/spi/README.md
// 
// SPI0, with two hardware chip selects, is available on the header of all Pis (although there is
//   an alternate mapping that is only usable on a Compute Module.
// SPI1, with three hardware chip selects, is available on 40-pin versions of Pis.
// SPI2, also with three hardware chip selects, is only usable on a Compute Module because the pins
//   aren't brought out onto the 40-pin header.
