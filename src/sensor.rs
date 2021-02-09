pub mod thermocouple;

mod mcp9600;
#[cfg(target = "armv7-unknown-linux-gnueabihf")]
pub use mcp9600::real::MCP9600;

#[cfg(not(target = "armv7-unknown-linux-gnueabihf"))]
pub use mcp9600::simulated::MCP9600;

mod heater;
#[cfg(target = "armv7-unknown-linux-gnueabihf")]
pub use heater::real::Heater;

#[cfg(not(target = "armv7-unknown-linux-gnueabihf"))]
pub use heater::simulated::Heater;
