use tracing::{event, span, Level};
use tracing_subscriber;

use warp::Filter;

mod config;
use config::Config;

mod kiln_control;
mod server;
use server::Manager;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let conf = Config::init().unwrap(); // TODO: Remove unwrap
    
    event!(Level::DEBUG, "system started");

    let manager = Manager::start();
    let manager = warp::any().map(move || manager.clone());

    let ws = warp::path("ws")
        .and(warp::ws())
        .and(manager)
        .map(|ws: warp::ws::Ws, manager: Manager| {
            ws.on_upgrade(move |socket| manager.on_connect(socket))
        });

    warp::serve(ws)
        .run(([127, 0, 0, 1], conf.web.port))
        .await;

    Ok(())
}

