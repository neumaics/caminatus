use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, UnboundedSender, Receiver};
use tokio::join;
use tracing::{debug, event, info, error, Level};
use uuid::Uuid;
use warp::ws::Message;
use warp::Error;

use crate::config::Config;
use crate::server::{Monitor, Command, Web};
use crate::device::Kiln;

type Subscriptions = Arc<Mutex<HashMap<String, Vec<(Uuid, UnboundedSender<Result<Message, Error>>)>>>>;

#[derive(Debug, Clone)]
pub struct Manager {
    sender: Sender<Command>,
}

impl Manager {
    pub async fn start() -> Result<Manager, String> {
        event!(Level::DEBUG, "system started");
        let (m_tx, m_rx) = mpsc::channel(16);
        tracing_subscriber::fmt::init();

        let conf = Config::init().unwrap(); // TODO: Remove unwrap
        let web = Web::start(conf.clone(), m_tx.clone());
        let subscriptions = Subscriptions::default();

        let monitor = Monitor::start(conf.poll_interval, m_tx.clone());
        
        tokio::spawn(async move { 
            let _ = Manager::process_commands(m_rx, subscriptions).await;
        });

        let kiln = Kiln::start(conf.poll_interval, m_tx.clone());
        
        join!(web, monitor, kiln);

        Ok(Manager {
            sender: m_tx,
        })
    }

    async fn process_commands(mut receiver: Receiver<Command>, subscriptions: Subscriptions) -> Result<(), String> {
        
        while let Some(command) = receiver.recv().await {
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
                        
                        let index = subs.iter().position(|s| s.0 == id);

                        if index.is_some() {
                            subs.remove(index.unwrap());
                        } else {
                            error!("attempting to unregister from channel not subscribed {}", id);
                        }
                        
                    } else {
                        error!("attempting to unsubscribe to a channel that doesn't exist");
                    }
                },
                Command::Register { channel } => {
                    debug!("registering service {}", channel);
                    let mut locked = subscriptions.lock().unwrap();

                    if locked.contains_key(&channel) {
                        debug!("channel already exists");
                    } else {
                        locked.insert(channel.to_string(), Vec::new());
                        debug!("channel doesn't exist, adding to list");
                    }
                },
                Command::Update { channel } => {
                    let mut locked = subscriptions.lock().unwrap();
                    debug!("updating the [{}] channel with new data", channel);

                    if locked.contains_key(&channel) {
                        let subs = locked.get_mut(&channel).unwrap();

                        for (_id, sender) in subs {
                            debug!("updating the user [id {}] with new data", _id);
                            
                            let _ = sender.send(Ok(Message::text("ping")));
                        }
                    } else {
                        debug!("attempting to update a channel that doesn't exist");
                    }
                },
                Command::Unknown { input } => {
                    error!("unknown command received {}. ignoring", input);
                },
                _ => {
                    debug!("ignoring command");
                }
            }
        }

        Ok(())
    }
}