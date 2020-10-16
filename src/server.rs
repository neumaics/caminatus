
use std::time::{Duration};

use uuid::Uuid;
use bytes::Bytes;

use tokio::sync::{watch, mpsc, oneshot};
use tokio::time;

use futures::{FutureExt, StreamExt};
use warp::ws::{Message, WebSocket};

use tracing::{debug, info};

type OneshotResponder<T> = oneshot::Sender<Result<T, warp::Error>>;

#[derive(Debug, Clone)]
struct Monitor {
    pub receiver: tokio::sync::watch::Receiver<&'static str>
}

impl Monitor {
    fn start() -> Monitor {
        let (status_tx, status_rx) = watch::channel("status");

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(10000));

            while let _ = interval.tick().await {
                let _ = status_tx.broadcast("beep");
            }
        });

        Monitor {
            receiver: status_rx
        }
    }
}

#[derive(Debug)]
enum Command {
    Subscribe {
        channel: String,
        responder: OneshotResponder<Bytes>,
    },
}

#[derive(Debug, Clone)]
pub struct Manager {
    monitor_service: Monitor,
    sender: tokio::sync::mpsc::Sender<Command>
}


impl Manager {
    pub fn start() -> Manager {
        let (m_tx, mut m_rx) = mpsc::channel(16);
    
        tokio::spawn(async move {
            while let Some(command) = m_rx.recv().await {
                match command {
                    Command::Subscribe { channel, responder } => {
                        //event!(Level::DEBUG, channel=channel.as_str(), "subscribing to channel");
                        let _ = responder.send(Ok(Bytes::from(Uuid::new_v4().to_string())));
                    }
                }
            }
        });

        Manager {
            sender: m_tx,
            monitor_service: Monitor::start()
        }
    }

    pub async fn on_connect(self, ws: WebSocket) {
        let (user_ws_tx, mut user_ws_rx) = ws.split();
        let mut monitor = self.monitor_service.receiver.clone();
        info!("client connecting");

        let (tx, rx) = mpsc::unbounded_channel();
        tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
            debug!("{:?}", result);
            if let Err(e) = result {
                eprintln!("websocket send error: {}", e);
            }
        }));
    
        tokio::task::spawn(async move { 
            while let Some(result) = monitor.recv().await {
                let _ = tx.send(Ok(Message::text(result)));
            }

            self.on_disconnect()
        });

        tokio::task::spawn(async move {
            while let Some(result) = user_ws_rx.next().await {
                let message = match result {
                    Ok(message) => message,
                    Err(error) => {
                        debug!("{:?}", error);
                        break;
                    }
                };
    
                debug!("{:?}", message);
            }

            // self.on_disconnect()
        });
    }

    async fn on_disconnect(self) {
        info!("client diconnecting");
    }
}
