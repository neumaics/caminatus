use caminatus::server::Manager;
use caminatus::Config;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let conf = Config::init()?;

    Manager::start(conf).await?;

    Ok(())
}
