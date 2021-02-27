use rppal::i2c;

#[derive(Debug)]
pub enum ThermocoupleError {
    UnsupportedPlatform { message: String },
    OpenCircuit,
    ShortCircuit,
    Unknown,
    I2CError { source: i2c::Error },
}

impl From<i2c::Error> for ThermocoupleError {
    fn from(error: i2c::Error) -> ThermocoupleError {
        ThermocoupleError::I2CError { source: error }
    }
}
