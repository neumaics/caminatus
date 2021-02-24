use anyhow::Result;
use tokio::sync::broadcast::Sender;
use warp::Filter;

use crate::server::Command;
use crate::config::Config;

pub mod error;

mod device;
mod schedules;
mod sse;
mod steps;

pub async fn start(conf: Config, manager_sender: Sender<Command>) -> Result<()> {
    let m3 = manager_sender.clone();

    let _ = tokio::spawn(async move {
        let public = warp::path("public")
            .and(warp::fs::dir("public"));

        let app = warp::path("app")
            .and(warp::filters::fs::file("public/index.html"));

        let index = warp::path::end()
            .and(warp::filters::fs::file("public/index.html"));

        let js = warp::path("bundle.js")
            .and(warp::filters::fs::file("public/bundle.js"));
        
        let routes = index
            .or(js)
            .or(public)
            .or(app)
            .or(sse::routes(&manager_sender))
            .or(device::routes(Some("./schedules".to_string()), m3))
            .or(schedules::routes(Some("./schedules".to_string())))
            .or(steps::routes());

        warp::serve(routes)
            .run((conf.web.host_ip, conf.web.port))
            .await;
    });

    Ok(())
}  
