use tokio;
use std::time::{Duration};
use tokio::time;

use caminatus::sensor::thermocouple::{Thermocouple, I2C};
use caminatus::sensor::mcp960x::MCP960X;

#[tokio::main]
async fn main() {
    let interval = 5000;
    let mut timer = time::interval(Duration::from_millis(interval as u64));

    loop {
        let thermocouple = MCP960X::new(0x00);
        let value = thermocouple.read();
        println!("the value is {:?}C", value);

        timer.tick().await;
    }
}