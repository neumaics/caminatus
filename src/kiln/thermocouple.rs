// FIXME: These are specific to the max31855 output. Make more generic.
pub enum ThermocoupleError {
    // When the thermocouple is open.
    // Bit D0, Default 0.
    Open_No_Connections,

    // When the thermocouple is short-circuited to GND (Ground).
    // Bit D1, Default 0.
    Short_To_Ground,

    // When the thermocouple is short-circuited to V_CC.
    // Bit D2, Default 0.
    Short_To_Vcc,
    
    // When SCV, SCG, or OC faults are active.
    // Bit D16, Default 0.
    Fault,
}

// TODO: Make measurement readings return Result<f64, ThermocoupleError>.
pub trait Thermocouple {

    fn new(clock_pin: u8, chip_select_pin: u8, master_out_slave_in_pin: u8) -> Self;
    fn begin(self) -> bool;
    fn read_internal(self) -> f64;
    fn read(self) -> f64;
    fn read_error(self) -> ThermocoupleError;
}

pub struct Simulated {
    initialized: bool,
    pub next_error: ThermocoupleError,
}

impl Thermocouple for Simulated {
    fn new(clock_pin: u8, cs_pin: u8, miso: u8) -> Self {
        Simulated {
            initialized: false,
            next_error: ThermocoupleError::Short_To_Vcc,
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
        self.next_error
    }
}

