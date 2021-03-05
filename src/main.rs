use std::path::PathBuf;
use structopt::StructOpt;

use caminatus::server::Manager;
use caminatus::Config;

use anyhow::Result;

#[derive(StructOpt, Debug)]
#[structopt(name = "caminatus")]
struct Opt {
    #[structopt(short, long, name = "CONFIG PATH", parse(from_os_str))]
    config_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt: Opt = Opt::from_args();
    let conf = Config::init(opt.config_path)?;

    Manager::start(conf).await?;

    Ok(())
}
