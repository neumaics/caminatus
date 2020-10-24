use tokio;
use tokio::time;

use std::time::{Duration};

use caminatus::sensor::mcp960x::MCP960X;

#[tokio::main]
async fn main() {
    let interval = 5000;
    let mut timer = time::interval(Duration::from_millis(interval as u64));

    loop {
        let thermocouple = MCP960X::new(0x60).unwrap();
        let value = thermocouple.read();
        println!("{:?} C", value);

        timer.tick().await;
    }
}