use std::panic;

use caminatus::sensor::MCP9600;

fn main() {
    match panic::catch_unwind(|| MCP9600::new(0x60)) {
        Ok(thermocouple) => {
            let value = thermocouple.unwrap().read();
            println!("{:?}C", value);
        },
        Err(error) => eprintln!("something went wrong. is this running on a pi? {:?}", error),
    };
}