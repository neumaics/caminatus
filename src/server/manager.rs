use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{Sender, UnboundedSender};

use tokio::join;
use tracing::{debug, event, error, Level, trace};
use uuid::Uuid;

use crate::config::Config;
use crate::server::{Message, Monitor, Command, Web};
// use crate::device::Kiln;

type SubscriptionList = Arc<Mutex<HashMap<String, Vec<(Uuid, UnboundedSender<Message>)>>>>;
type ServiceList = Arc<Mutex<HashMap<String, Sender<Command>>>>;

#[derive(Debug, Clone)]
pub struct Manager {
    sender: broadcast::Sender<Command>,
}

impl Manager {
    pub async fn start(conf: Config) -> Result<Manager> {
        event!(Level::DEBUG, "system starting");
        let (b_tx, b_rx): (broadcast::Sender<Command>, broadcast::Receiver<Command>) = broadcast::channel(32);
        tracing_subscriber::fmt::init();

        let web = Web::start(conf.clone(), b_tx.clone());
        let subscriptions = SubscriptionList::default();
        let services = ServiceList::default();

        let monitor = Monitor::start(conf.poll_interval, b_tx.clone());

        tokio::task::spawn(async move {
            let _ = Manager::process_commands(b_rx, subscriptions, services).await;
        });

        // todo: use value of join
        let _ = join!(web, monitor);

        Ok(Manager {
            sender: b_tx,
        })
    }

    async fn process_commands(
        mut receiver: broadcast::Receiver<Command>,
        subscriptions: SubscriptionList,
        _services: ServiceList
    ) -> Result<()> {
        while let Ok(command) = receiver.recv().await {
            match command {
                Command::Subscribe { channel, id, sender } => {
                    trace!("subscribing to channel {}", channel);
                    let mut locked = subscriptions.lock().unwrap();
                    let temp = sender.clone();

                    if locked.contains_key(&channel) {
                        let subs = locked.get_mut(&channel).unwrap();
                        subs.push((id, sender));
                        let _ = temp.send(Message::Update("success".to_string()));
                    } else {
                        trace!("attempting to subscribe to a channel that doesn't exist");
                    }                    
                },
                Command::Unsubscribe { channel, id } => {
                    trace!("unsubscribing to channel {}", channel);
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
                        debug!("channel [{}] already exists", channel);
                    } else {
                        debug!("channel [{}] doesn't exist, adding to list", channel);
                        locked.insert(channel.to_string(), Vec::new());
                    }
                },
                Command::Update { channel, data } => {
                    let mut locked = subscriptions.lock().unwrap();
                    trace!("updating the [{}] channel with new data", channel);

                    if locked.contains_key(&channel) {
                        locked.get_mut(&channel).unwrap().retain(|(id, sender)| {
                            trace!("updating the user [id {}] with new data", id);
                            sender.send(Message::Update(data.clone())).is_ok()
                        });
                    } else {
                        error!("attempting to update a channel that doesn't exist");
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
