use tokio::sync::mpsc;
use tracing::{debug, info};

use tracing::{event, Level};

use crate::config::Config;
use crate::server::{Monitor, Command, Web};


#[derive(Debug, Clone)]
pub struct Manager {
    monitor_service: Monitor,
    sender: tokio::sync::mpsc::Sender<Command>
}

impl Manager {
    pub async fn start() -> Result<Manager, String> {
        event!(Level::DEBUG, "system started");
        let (m_tx, mut m_rx) = mpsc::channel(16);
        tracing_subscriber::fmt::init();

        let conf = Config::init().unwrap(); // TODO: Remove unwrap
        let web_box = Web::start();
    
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

        let _ = web_box.unwrap().task_handle.await;

        Ok(Manager {
            sender: m_tx,
            monitor_service: Monitor::start(conf.poll_interval)
        })
        
    }
}