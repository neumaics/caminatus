
use futures::{FutureExt, StreamExt};
use tokio::{task, join};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tracing::{debug, info};
use uuid::Uuid;
use warp::ws::{WebSocket};
use warp::Filter;

use crate::server::{Command, Api};
use crate::config::Config;

pub struct Web {
    pub sender: Sender<Command>,
    pub task_handle: tokio::task::JoinHandle<()>, // FIXME: This feels especially inelegant.
}

impl Web {
    pub fn start(conf: Config, manager_sender: Sender<Command>) -> Result<Self, String> {
        let (tx, mut rx) = mpsc::channel(16);
        
        let manager = warp::any().map(move || manager_sender.clone());

        let task_handle = tokio::spawn(async move {
            let ws = warp::path("ws")
                .and(warp::ws())
                .and(manager)
                .map(|ws: warp::ws::Ws, manager: Sender<Command>| {
                    ws.on_upgrade(move |socket| on_connect(manager.clone(), socket))
                });

            let public = warp::path("public")
                .and(warp::fs::dir("public"));

            let routes = ws.or(public);

            warp::serve(routes)
                .run(([127, 0, 0, 1], conf.web.port))
                .await;
        });

        Ok(Web {
            sender: tx,
            task_handle: task_handle
        })
    }
}

async fn on_connect(manager: Sender<Command>, ws: WebSocket) {
    let id = Uuid::new_v4();
    let (user_ws_tx, mut user_ws_rx) = ws.split();
    // let copy1 = manager.clone();
    let mut copy2 = manager.clone();
    // let copy3 = manager.clone();
    // let _monitor = copy1.monitor_service.receiver.clone();

    info!("client connecting");
    let (tx, rx) = mpsc::unbounded_channel();
    let forwarder = tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
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
                    Api::Subscribe { channel } => Command::Subscribe {
                        channel: channel,
                        id: id,
                        sender: cmd_tx.clone()
                    },
                    Api::Unsubscribe { channel } => Command::Unsubscribe {
                        channel: channel,
                        id: id,
                    },
                    Api::Unknown { input } => Command::Unknown { input: input }
                };
                // match 
                let _ = copy2.send(command).await;
            }
        }

        let _ = on_disconnect();
    });

    join!(command_reader, forwarder);
}

async fn on_disconnect() {
    info!("client diconnecting");
}