use tokio;
use tokio::time;

use std::time::Duration;

use caminatus::sensor::MCP9600;

#[tokio::main]
async fn main() {
    let interval = 5000;
    let mut timer = time::interval(Duration::from_millis(interval as u64));
    let mut thermocouple = MCP9600::new(0x60).unwrap();

    loop {
        let value = thermocouple.read();
        println!("{:?} C", value);

        timer.tick().await;
    }
}
