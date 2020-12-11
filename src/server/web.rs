use futures::{FutureExt, StreamExt};
use serde_json;
use tokio::{task, join};
use tokio::sync::mpsc;
use tokio::sync::broadcast::Sender;
use tracing::{debug, error, info};
use uuid::Uuid;
use warp::ws::{WebSocket};
use warp::{filters::BoxedFilter, Filter, Reply};

use crate::server::{Command, Api};
use crate::config::Config;
use crate::schedule::Schedule;

pub struct Web { }

impl Web {
    pub async fn start(conf: Config, manager_sender: Sender<Command>) -> Result<Self, String> {
        let manager = warp::any().map(move || manager_sender.clone());

        let _ = tokio::spawn(async move {
            let ws = warp::path("ws")
                .and(warp::ws())
                .and(manager)
                .map(|ws: warp::ws::Ws, manager: Sender<Command>| {
                    ws.on_upgrade(move |socket| on_connect(manager, socket))
                });

            let public = warp::path("public")
                .and(warp::fs::dir("public"));

            let routes = ws
                .or(public)
                .or(Web::schedule_routes());

            warp::serve(routes)
                .run((conf.web.host_ip, conf.web.port))
                .await;
        }).await;

        Ok(Web { })
    }

    fn schedule_routes() -> BoxedFilter<(impl Reply,)> {
        let schedules = warp::get()
            .and(warp::path("schedules"))
            .and(warp::path::end())
            .map(|| serde_json::to_string(&Schedule::all()).unwrap());
        
        let schedule = warp::get()
            .and(warp::path("schedules"))
            .and(warp::path::param())
            .map(|schedule_name: String| serde_json::to_string(&Schedule::by_name(schedule_name)).unwrap());

        let new_schedule = warp::post()
            .and(warp::path("schedules"))
            .and(warp::path::end())
            .and(warp::body::content_length_limit(1024 * 32))
            .and(warp::body::json())
            .map(|body: Schedule| Schedule::new(body).unwrap());

        let update_schedule = warp::put()
            .and(warp::path("schedules"))
            .and(warp::path::param())
            .and(warp::path::end())
            .and(warp::body::content_length_limit(1024 * 32))
            .and(warp::body::json())
            .map(|schedule_name: String, body: Schedule| Schedule::update(schedule_name, body).unwrap());
        
        let delete_schedule = warp::delete()
            .and(warp::path("schedules"))
            .and(warp::path::param())
            .and(warp::path::end())
            .map(|schedule_id: String| Schedule::delete(schedule_id).unwrap());

        schedules
            .or(schedule)
            .or(new_schedule)
            .or(update_schedule)
            .or(delete_schedule)
            .boxed()
    }
}

async fn on_connect(manager: Sender<Command>, ws: WebSocket) {
    let id = Uuid::new_v4();
    let (user_ws_tx, mut user_ws_rx) = ws.split();
    let copy2 = manager.clone();

    info!("client connecting");
    let (tx, rx) = mpsc::unbounded_channel();
    let forwarder = task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            error!("websocket send error: {}", e);
        }
    }));

    let cmd_tx = tx.clone();
    let command_reader = task::spawn(async move {
        while let Some(result) = user_ws_rx.next().await {
            debug!("{:?}", result);
            let message = match result {
                Ok(message) => message,
                Err(error) => {
                    debug!("{:?}", error);
                    break;
                }
            };
            
            if message.to_str().is_ok() {
                let api: Api = Api::from(message.to_str().unwrap());

                let command = match api {
                    Api::Schedules => Command::Unknown { input: "schedules".to_string() },
                    Api::Subscribe { channel } => Command::Subscribe {
                        channel: channel,
                        id: id,
                        sender: cmd_tx.clone()
                    },
                    Api::Unsubscribe { channel } => Command::Unsubscribe {
                        channel: channel,
                        id: id,
                    },
                    Api::Start { schedule_name } => Command::Forward {
                        channel: "client".to_string(),
                        cmd: Box::new(Command::Start {
                            simulate: false,
                            schedule: Schedule::by_name(schedule_name).unwrap()
                    })},
                    Api::Unknown { input } => Command::Unknown { input: input },
                };
                let _ = copy2.send(command);
            }
        }

        let _ = on_disconnect();
    });

    join!(command_reader, forwarder);
}

async fn on_disconnect() {
    info!("client disconnecting");
}
