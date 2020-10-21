use caminatus::server::Manager;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _ = Manager::start().await;

    Ok(())
}
