use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, UnboundedSender};
use tokio::{task, join};
use tracing::{event, info, debug, Level};
use uuid::Uuid;
use warp::ws::Message;
use warp::Error;

use crate::config::Config;
use crate::server::{Monitor, Command, Web, Api};

type Subscriptions = Arc<Mutex<HashMap<String, Vec<(Uuid, UnboundedSender<Result<Message, Error>>)>>>>;

#[derive(Debug, Clone)]
pub struct Manager {
    sender: Sender<Command>
}

impl Manager {
    pub async fn start() -> Result<Manager, String> {
        event!(Level::DEBUG, "system started");
        let (m_tx, mut m_rx) = mpsc::channel(16);
        tracing_subscriber::fmt::init();

        let conf = Config::init().unwrap(); // TODO: Remove unwrap
        let web_box = Web::start(conf.clone(), m_tx.clone());
        let subscriptions = Subscriptions::default();

        let monitor = Monitor::start(conf.poll_interval, m_tx.clone());
    
        tokio::spawn(async move {
            while let Some(command) = m_rx.recv().await {
                match command {
                    Command::Subscribe { channel, id, sender } => {
                        info!("subscribing to channel {}", channel);
                        let mut locked = subscriptions.lock().unwrap();

                        if locked.contains_key(&channel) {
                            let subs = locked.get_mut(&channel).unwrap();
                            subs.push((id, sender));
                        } else {
                            info!("attempting to subscribe to a channel that doesn't exist");
                        }
                    },
                    Command::Unsubscribe { channel, id } => {
                        info!("unsubscribing to channel {}", channel);
                        let mut locked = subscriptions.lock().unwrap();

                        if locked.contains_key(&channel) {
                            let subs = locked.get_mut(&channel).unwrap();
                            let index = subs.iter().position(|s| s.0 == id).unwrap();
                            subs.remove(index);
                        } else {
                            info!("attempting to unsubscribe to a channel that doesn't exist");
                        }
                    },
                    Command::Register { channel } => {
                        info!("registering service {}", channel);
                        let mut locked = subscriptions.lock().unwrap();
                        // let foo = .contains_key(&channel);
                        if locked.contains_key(&channel) {
                            info!("channel already exists");
                        } else {
                            locked.insert(channel.to_string(), Vec::new());
                            info!("channel doesn't exist, adding to list");
                        }
                    },
                    Command::Update { channel } => {
                        let mut locked = subscriptions.lock().unwrap();
                        info!("updating the [{}] channel with new data", channel);

                        if locked.contains_key(&channel) {
                            let subs = locked.get_mut(&channel).unwrap();
                            // subs.push((id, sender));
                            for (_id, sender) in subs {
                                info!("updating the user [id {}] with new data", _id);
                                // if let Err(_) = sender.send(Ok(Message::text("ping"))) {
                                //     debug!("error sending update to client");
                                // };
                                let _ = sender.send(Ok(Message::text("ping")));
                            }
                        } else {
                            info!("attempting to update a channel that doesn't exist");
                        }
                    },
                    Command::Unknown { input } => {
                        info!("unknown command received {}", input);
                    }
                }
            }
        });
        
        join!(web_box.unwrap().task_handle, monitor);

        Ok(Manager {
            sender: m_tx,
            // monitor_service: Monitor::start(conf.poll_interval)
        })
        
    }
}