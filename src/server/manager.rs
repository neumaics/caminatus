use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;
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

type SubscriptionList = Arc<Mutex<HashMap<String, Vec<(Uuid, UnboundedSender<Result<Message, Error>>)>>>>;
type ServiceList = Arc<Mutex<HashMap<String, Sender<Command>>>>;

#[derive(Debug, Clone)]
pub struct Manager {
    sender: Sender<Command>,
}

impl Manager {
    pub async fn start() -> Result<Manager> {
        event!(Level::DEBUG, "system started");
        let (m_tx, m_rx) = mpsc::channel(16);
        tracing_subscriber::fmt::init();

        let conf = Config::init()?;
        let web = Web::start(conf.clone(), m_tx.clone());
        let subscriptions = SubscriptionList::default();
        let services = ServiceList::default();

        let monitor = Monitor::start(conf.poll_interval, m_tx.clone());
        let kiln = Kiln::start(conf.poll_interval, m_tx.clone()).await?;

        
        tokio::spawn(async move {
            let _ = Manager::process_commands(m_rx, subscriptions, services).await;
        });

        join!(web, monitor);

        Ok(Manager {
            sender: m_tx,
        })
    }

    async fn process_commands(
        mut receiver: Receiver<Command>,
        subscriptions: SubscriptionList,
        services: ServiceList
    ) -> Result<()> {
        
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
                        debug!("channel [{}] already exists", channel);
                    } else {
                        debug!("channel [{}] doesn't exist, adding to list", channel);
                        locked.insert(channel.to_string(), Vec::new());
                    }
                },
                Command::Update { channel, data } => {
                    let mut locked = subscriptions.lock().unwrap();
                    debug!("updating the [{}] channel with new data", channel);

                    if locked.contains_key(&channel) {
                        let subs = locked.get_mut(&channel).unwrap();

                        for (_id, sender) in subs {
                            debug!("updating the user [id {}] with new data", _id);
                            
                            let _ = sender.send(Ok(Message::text(data.clone())));
                        }
                    } else {
                        debug!("attempting to update a channel that doesn't exist");
                    }
                },
                Command::Start { schedule, simulate } => {

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