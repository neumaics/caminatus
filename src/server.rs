use std::time::{Duration};

use tokio::sync::{watch, mpsc};
use tokio::time;

use futures::{FutureExt, StreamExt};
use warp::ws::{WebSocket};

use tracing::{debug, info};

use crate::config::Config;

mod command;
use command::Command;

#[derive(Debug, Clone)]
struct Monitor {
    pub name: &'static str,
    pub receiver: tokio::sync::watch::Receiver<&'static str>
}

impl Monitor {
    fn start(interval: u32) -> Monitor {
        let name = "status";
        let (status_tx, status_rx) = watch::channel(name);
        let mut interval = time::interval(Duration::from_millis(interval as u64));

        tokio::spawn(async move {
            interval.tick().await;
            let _ = status_tx.broadcast("beep");
        });

        Monitor {
            name: name,
            receiver: status_rx
        }
    }
}


#[derive(Debug, Clone)]
pub struct Manager {
    monitor_service: Monitor,
    sender: tokio::sync::mpsc::Sender<Command>
}

impl Manager {
    pub fn start(config: &Config) -> Manager {
        let (m_tx, mut m_rx) = mpsc::channel(16);
    
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
            monitor_service: Monitor::start(config.poll_interval)
        }
    }

    pub async fn on_connect(self, ws: WebSocket) {
        let (user_ws_tx, mut user_ws_rx) = ws.split();
        let copy1 = self.clone();
        let mut copy2 = self.clone();
        let copy3 = self.clone();
        let mut monitor = copy1.monitor_service.receiver.clone();

        info!("client connecting");

        let (tx, rx) = mpsc::unbounded_channel();
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
