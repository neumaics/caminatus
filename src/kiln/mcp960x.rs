/// Thermocouple EMF to Temperature Converter
/// http://ww1.microchip.com/downloads/en/DeviceDoc/MCP960X-L0X-RL0X-Data-Sheet-20005426F.pdf
/// https://www.adafruit.com/product/4101
use rppal::i2c::I2c;

use crate::kiln::thermocouple::{Thermocouple, ThermocoupleError};

pub struct MCP960X {

}

impl Thermocouple for MCP960X {
    fn new(clock_pin: u8, cs_pin: u8, miso: u8) -> Self {
        MCP960X {}
    }

    fn begin(self) -> bool {
        // self.initialized = true;
        true
    } 

    fn read_internal(self) -> f64 {
        0.0
    }

    fn read(self) -> f64 {
        0.0
    }

    fn read_error(self) -> ThermocoupleError {
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
