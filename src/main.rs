use tracing::{event, Level};
use tracing_subscriber;

use warp::Filter;

mod config;
use config::Config;

mod server;
use server::Manager;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let conf = Config::init().unwrap(); // TODO: Remove unwrap
    
    event!(Level::DEBUG, "system started");

    let manager = Manager::start(&conf);
    let manager = warp::any().map(move || manager.clone());

    let ws = warp::path("ws")
        .and(warp::ws())
        .and(manager)
        .map(|ws: warp::ws::Ws, manager: Manager| {
            ws.on_upgrade(move |socket| manager.on_connect(socket))
        });

    // let public = warp::path::end()
    let public = warp::path("public")
        .and(warp::fs::dir("public"));

    let routes = ws.or(public);

    warp::serve(routes)
        .run(([127, 0, 0, 1], conf.web.port))
        .await;
    
    Ok(())
}
