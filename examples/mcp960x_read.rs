use std::panic;

use caminatus::sensor::mcp960x::MCP960X;

fn main() {
    match panic::catch_unwind(|| MCP960X::new(0x60)) {
        Ok(thermocouple) => {
            let value = thermocouple.unwrap().read();
            println!("{:?}C", value);
        },
        Err(error) => eprintln!("something went wrong. is this running on a pi? {:?}", error),
    };
}