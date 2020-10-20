pub enum ThermocoupleError {
    UNSUPPORTED_PLATFORM {
        message: String,
    },
    OPEN_CIRCUIT,
    SHORT_CIRCUIT,
    UNKNOWN,
}

// TODO: Make measurement readings return Result<f64, ThermocoupleError>.
pub trait Thermocouple {
    fn read_internal(self) -> f64;
    fn read(self) -> f64;
    fn read_error(self) -> ThermocoupleError;
}

pub trait I2C {
    fn new(address: u16) -> Self;
}

pub trait SPI {
    fn new(clock_pin: u8, chip_select_pin: u8, master_in_slave_out_pin: u8) -> Self;
}

pub struct Simulated {
    pub next_error: ThermocoupleError,
    pub next_internal: f64,
    pub next_hotend: f64,
}

impl SPI for Simulated {
    fn new(clock_pin: u8, cs_pin: u8, miso: u8) -> Self {
        Simulated {
            next_internal: 0.0,
            next_hotend: 0.0,
            next_error: ThermocoupleError::UNKNOWN,
        }
    }
}

impl Thermocouple for Simulated {

    fn read_internal(self) -> f64 {
        self.next_internal
    }

    fn read(self) -> f64 {
        self.next_hotend
    }

    fn read_error(self) -> ThermocoupleError {
        self.next_error
    }
}