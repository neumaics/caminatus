use caminatus::kiln::thermocouple::{Thermocouple, I2C};
use caminatus::kiln::mcp960x::MCP960X;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let thermocouple = MCP960X::new(0x00);

    // loop {
    //     let temperature = thermocouple.read();
    // }
    
    Ok(())
}