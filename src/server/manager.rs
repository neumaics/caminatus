use futures::{FutureExt, StreamExt};
use warp::ws::{WebSocket};
use warp::Filter;

use tokio::sync::mpsc;
use tracing::{debug, info};

use tracing::{event, Level};

use crate::config::Config;
use crate::server::monitor::Monitor;
use crate::server::command::Command;

#[derive(Debug, Clone)]
pub struct Manager {
    monitor_service: Monitor,
    sender: tokio::sync::mpsc::Sender<Command>
}

impl Manager {
    pub fn start() -> Manager {
        event!(Level::DEBUG, "system started");
        let (m_tx, mut m_rx) = mpsc::channel(16);
        tracing_subscriber::fmt::init();

        let conf = Config::init().unwrap(); // TODO: Remove unwrap

    // let ws = warp::path("ws")
    //     .and(warp::ws())
    //     .and(manager)
    //     .map(|ws: warp::ws::Ws, manager: Manager| {
    //         ws.on_upgrade(move |socket| manager.on_connect(socket))
    //     });

    // // let public = warp::path::end()
    // let public = warp::path("public")
    //     .and(warp::fs::dir("public"));

    // let routes = ws.or(public);

    // warp::serve(routes)
    //     .run(([127, 0, 0, 1], conf.web.port))
    //     .await;
    
        tokio::spawn(async move {
            while let Some(command) = m_rx.recv().await {
                match command {
                    Command::Subscribe { channel } => {
                        info!("subscribing to channel {}", channel);
                        //event!(Level::DEBUG, channel=channel.as_str(), "subscribing to channel");
                        // let _ = responder.send(Ok(Bytes::from(Uuid::new_v4().to_string())));
                    },
                    Command::Unsubscribe { channel } => {
                        info!("unsubscribing to channel {}", channel);
                    },
                    Command::Unknown { input } => {
                        info!("unknown command received {}", input);
                    }
                }
            }
        });

        Manager {
            sender: m_tx,
            monitor_service: Monitor::start(conf.poll_interval)
        }
    }

    pub async fn on_connect(self, ws: WebSocket) {
        let (user_ws_tx, mut user_ws_rx) = ws.split();
        let copy1 = self.clone();
        let mut copy2 = self.clone();
        let copy3 = self.clone();
        let _monitor = copy1.monitor_service.receiver.clone();

        info!("client connecting");

        let (_tx, rx) = mpsc::unbounded_channel();
        tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
            debug!("{:?}", result);
            if let Err(e) = result {
                eprintln!("websocket send error: {}", e);
            }
        }));
    
        // tokio::task::spawn(async move { 
        //     while let Some(result) = monitor.recv().await {
        //         let _ = tx.send(Ok(Message::text(result)));
        //     }

        //     copy1.on_disconnect()
        // });

        tokio::task::spawn(async move {
            while let Some(result) = user_ws_rx.next().await {
                debug!("{:?}", result);
                let message = match result {
                    Ok(message) => message,
                    Err(error) => {
                        debug!("{:?}", error);
                        break;
                    }
                };
                
                let command: Command = Command::from(message.to_str().unwrap());

                let _ = copy2.sender.send(command).await;
            }

            let _ = copy3.on_disconnect();
        });
    }

    async fn on_disconnect(self) {
        info!("client diconnecting");
    }
}