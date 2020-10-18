use std::panic;

use caminatus::kiln::thermocouple::{Thermocouple, I2C};
use caminatus::kiln::mcp960x::MCP960X;

fn main() {
    match panic::catch_unwind(|| MCP960X::new(0x00)) {
        Ok(thermocouple) => {
            let value = thermocouple.read();
            println!("the value is {:?}C", value);
        },
        Err(error) => eprintln!("something went wrong. is this running on a pi? {:?}", error),
    };
}