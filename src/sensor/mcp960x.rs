/// 
/// Thermocouple EMF to Temperature Converter
/// Datasheet:
///   http://ww1.microchip.com/downloads/en/DeviceDoc/MCP960X-L0X-RL0X-Data-Sheet-20005426F.pdf
/// 
/// Sample breakout board:
///   https://www.adafruit.com/product/4101
/// 
use rppal::i2c::I2c;
use tracing::error;

use crate::sensor::thermocouple::ThermocoupleError;

// Registers
const HOT_JUNCTION_TEMPERATURE: u8 = 0x00;
// const JUNCTION_TEMPERATURE_DELTA: u8 = 0x01;
const COLD_JUNCTION_TEMPERATURE: u8 = 0x02;
// const RAW_DATA: u8 = 0x03;
// const STATUS: u8 = 0x04;
// const SENSOR_CONFIGURATION: u8 = 0x05;
// const DEVICE_CONFIGURATION: u8 = 0x06;
// const ALERT_1_CONFIGURATION: u8 = 0x08;
// const ALERT_2_CONFIGURATION: u8 = 0x09;
// const ALERT_3_CONFIGURATION: u8 = 0x0A;
// const ALERT_4_CONFIGURATION: u8 = 0x0B;
// const ALERT_1_HYSTERESIS: u8 = 0x0C;
// const ALERT_2_HYSTERESIS: u8 = 0x0D;
// const ALERT_3_HYSTERESIS: u8 = 0x0E;
// const ALERT_4_HYSTERESIS: u8 = 0x0F;
// const ALERT_1_LIMIT: u8 = 0x10;
// const ALERT_2_LIMIT: u8 = 0x11;
// const ALERT_3_LIMIT: u8 = 0x12;
// const ALERT_4_LIMIT: u8 = 0x13;
// const DEVICE_ID: u8 = 0x20;

// Hot-junction and alert temperatures use the first bit of the upper byte as the sign.
const FIRST_BIT_SIGN: u8 = 0x7F;

// Cold-junction temperatures use the first four bits of the upper byte as the sign.
const TOP_HALF_SIGN: u8 = 0x0F;

// The Raw Data ADC register uses the first six bits of the upper byte as the sign.
const _DATA_SIGN: u8 = 0x03;


pub struct MCP960X {
    address: u16,
    i2c: I2c
}

impl MCP960X {
    pub fn new(address: u16) -> Result<Self, ThermocoupleError> {
        let mut i2c = I2c::new()?;
        
        match i2c.set_slave_address(address) {
            Ok(_) => Ok(MCP960X {
                address,
                i2c: i2c,
            }),
            Err(error) => Err(ThermocoupleError::I2CError { source: error })
        }   
    }

    pub fn read_internal(self) -> Result<f64, ThermocoupleError> {
        self.read_temperature(COLD_JUNCTION_TEMPERATURE, TOP_HALF_SIGN)
    }

    pub fn read(self) -> Result<f64, ThermocoupleError> {
        self.read_temperature(HOT_JUNCTION_TEMPERATURE, FIRST_BIT_SIGN)
    }

    fn read_temperature(mut self, junction: u8, sign_bits: u8) -> Result<f64, ThermocoupleError> {
        let mut register = [0u8; 2];

        let write_command: u8 = (self.address << 1) as u8;
        let read_command:u8 = write_command | 0x01;
        
        self.i2c.block_write(write_command, &[junction])?;
        self.i2c.write(&[read_command])?;

        let result = self.i2c.block_read(junction, &mut register);

        match result {
            Ok(()) => Ok(to_float(register, sign_bits)),
            Err(error) => {
                error!("error: {}", error);
                Err(ThermocoupleError::I2CError { source: error })
            },
        }
    }

    pub fn read_error(self) -> ThermocoupleError {
        // TODO: Read status register and return errors.
        ThermocoupleError::Unknown
    }
}


/// Converts the two byte representation of the temperature to its floating point representation.
///   See in the datasheet: TABLE 5-1:SUMMARY OF REGISTERS AND BIT ASSIGNMENTS
/// 
/// Hot junction and junctions temperature delta, alert limits registers:
///   |       | bit 7 | bit 6 | bit 5 | bit 4 | bit 3 | bit 2 |  bit 1 |   bit 0 |
///   |-------|-------|-------|-------|-------|-------|-------|--------|---------|
///   | upper |  SIGN | 1024C |  512C |  256C |  128C |   64C |    32C |     16C |
///   | lower |    8C |    4C |    2C |    1C |  0.5C | 0.25C | 0.125C | 0.0625C |
/// 
/// Cold junction temperature
///   | upper |  SIGN |  SIGN |  SIGN |  SIGN |  128C |   64C |    32C |     16C |
///   | lower |    8C |    4C |    2C |    1C |  0.5C | 0.25C | 0.125C | 0.0625C |
fn to_float(register: [u8; 2], sign_mask: u8) -> f64 {
    let [upper, lower] = register;
    let sign: f64 = if (upper.clone() & 0x80) == 0 { 1.0 } else { -1.0 };

    let upper = upper & sign_mask;
    let upper: u16 = (upper as u16) << 4;
    let lower: u16 = (lower as u16) << 4;

    let lower_div: f64 = (lower as f64) / 256.0;
    let result = (upper as f64) + lower_div;
    
    sign * result
}

impl Clone for MCP960X {
    fn clone(&self) -> MCP960X {
        MCP960X::new(self.address).unwrap()
    }
}

#[cfg(test)]
mod to_float_tests {
    use super::*;

    fn test_hot_to_float(upper: u8, lower: u8, expected_output: f64) {
        let input: [u8; 2] = [upper, lower];
        let output: f64 = to_float(input, FIRST_BIT_SIGN);

        assert_eq!(output, expected_output);
    }

    fn test_cold_to_float(upper: u8, lower: u8, expected_output: f64) {
        let input: [u8; 2] = [upper, lower];
        let output: f64 = to_float(input, TOP_HALF_SIGN);

        assert_eq!(output, expected_output);
    }
    
    #[test]
    fn works() {
        // Sanity check
        assert_eq!(to_float([0u8; 2], FIRST_BIT_SIGN), 0.0);
    }

    #[test]
    fn can_get_the_correct_sign() {
        let input: [u8; 2] = [0b0111_1111, 0b0000_0000];
        let output: f64 = to_float(input, FIRST_BIT_SIGN);

        assert!(output > 0.0, "result was less than zero");

        let input: [u8; 2] = [0b1111_1111, 0b0000_0000];
        let output: f64 = to_float(input, FIRST_BIT_SIGN);

        assert!(output < 0.0, "result was greater than zero");
    }

    #[test]
    fn can_convert_hot_temperatures_correctly() {
        // Test every individual bit, to start.
        test_hot_to_float(0b0000_0000, 0b0000_0000, 0.0);
        test_hot_to_float(0b0000_0000, 0b0000_0001, 0.0625);
        test_hot_to_float(0b0000_0000, 0b0000_0010, 0.125);
        test_hot_to_float(0b0000_0000, 0b0000_0100, 0.25);
        test_hot_to_float(0b0000_0000, 0b0000_1000, 0.5);
        test_hot_to_float(0b0000_0000, 0b0001_0000, 1.0);
        test_hot_to_float(0b0000_0000, 0b0010_0000, 2.0);
        test_hot_to_float(0b0000_0000, 0b0100_0000, 4.0);
        test_hot_to_float(0b0000_0000, 0b1000_0000, 8.0);

        test_hot_to_float(0b0000_0001, 0b0000_0000, 16.0);
        test_hot_to_float(0b0000_0010, 0b0000_0000, 32.0);
        test_hot_to_float(0b0000_0100, 0b0000_0000, 64.0);
        test_hot_to_float(0b0000_1000, 0b0000_0000, 128.0);
        test_hot_to_float(0b0001_0000, 0b0000_0000, 256.0);
        test_hot_to_float(0b0010_0000, 0b0000_0000, 512.0);
        test_hot_to_float(0b0100_0000, 0b0000_0000, 1024.0);
        test_hot_to_float(0b1100_0000, 0b0000_0000, -1024.0);
        
        // -0, actually. not sure if this is a valid output
        test_hot_to_float(0b1000_0000, 0b0000_0000, 0.0);        
    }

    #[test]
    fn can_convert_cold_temperatures_correctly() {
        test_cold_to_float(0b0000_0000, 0b0000_0000, 0.0);
        test_cold_to_float(0b0000_0000, 0b0000_0001, 0.0625);
        test_cold_to_float(0b0000_0000, 0b0000_0010, 0.125);
        test_cold_to_float(0b0000_0000, 0b0000_0100, 0.25);
        test_cold_to_float(0b0000_0000, 0b0000_1000, 0.5);
        test_cold_to_float(0b0000_0000, 0b0001_0000, 1.0);
        test_cold_to_float(0b0000_0000, 0b0010_0000, 2.0);
        test_cold_to_float(0b0000_0000, 0b0100_0000, 4.0);
        test_cold_to_float(0b0000_0000, 0b1000_0000, 8.0);

        test_cold_to_float(0b0000_0001, 0b0000_0000, 16.0);
        test_cold_to_float(0b0000_0010, 0b0000_0000, 32.0);
        test_cold_to_float(0b0000_0100, 0b0000_0000, 64.0);
        test_cold_to_float(0b0000_1000, 0b0000_0000, 128.0);

        // -0, actually. not sure if this is a valid output
        test_cold_to_float(0b0001_0000, 0b0000_0000, 0.0);
        test_cold_to_float(0b0010_0000, 0b0000_0000, 0.0);
        test_cold_to_float(0b0100_0000, 0b0000_0000, 0.0);
        test_cold_to_float(0b1000_0000, 0b0000_0000, 0.0);   
    }

    // TODO: Test every possible value...?
}