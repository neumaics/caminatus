use caminatus::Config;
use caminatus::server::Manager;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let conf = Config::init()?;

    Manager::start(conf).await?;

    Ok(())
}
