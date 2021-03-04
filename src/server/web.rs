use tokio::sync::broadcast::Sender;
use warp::Filter;

use crate::config::Config;
use crate::server::Command;

pub mod error;

mod device;
mod schedules;
mod sse;
mod static_file;
mod steps;

pub async fn start(conf: Config, manager_sender: Sender<Command>) {
    tokio::spawn(async move {
        let routes = static_file::routes()
            .or(sse::routes(&manager_sender))
            .or(device::routes(
                Some("./schedules".to_string()),
                &manager_sender,
            ))
            .or(schedules::routes(Some("./schedules".to_string())))
            .or(steps::routes());

        warp::serve(routes)
            .run((conf.web.host_ip, conf.web.port))
            .await;
    });
}
