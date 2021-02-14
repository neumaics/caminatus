use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{Sender, UnboundedSender};
use tokio::join;
use tracing::{debug, error, instrument, trace};
use uuid::Uuid;

use crate::config::Config;
use crate::server::{Message, Monitor, Command, Web};
use crate::device::{Kiln, KilnEvent};

type SubscriptionList = Arc<Mutex<HashMap<String, Vec<(Uuid, UnboundedSender<Message>)>>>>;
type ServiceList = Arc<Mutex<HashMap<String, Sender<Command>>>>;
type ClientList = Arc<Mutex<HashMap<Uuid, UnboundedSender<Message>>>>;

#[derive(Debug, Clone)]
pub struct Manager {
    sender: broadcast::Sender<Command>,
}

impl Manager {
    
    #[instrument]
    pub async fn start(conf: Config) -> Result<Manager> {
        debug!("starting manager");
        let (b_tx, b_rx): (broadcast::Sender<Command>, broadcast::Receiver<Command>) = broadcast::channel(32);
        tracing_subscriber::fmt::init();

        let web = Web::start(conf.clone(), b_tx.clone());
        let kiln = Kiln::start(conf.thermocouple_address, conf.gpio.heater, conf.poll_interval, b_tx.clone()).await?;
        let subscriptions = SubscriptionList::default();
        let services = ServiceList::default();
        let clients = ClientList::default();

        let _monitor = Monitor::start(conf.web.keep_alive_interval, b_tx.clone());

        let proc = tokio::task::spawn(async move {
            let _ = Manager::process_commands(b_rx, subscriptions, services, clients, kiln).await;
        });

        let _ = join!(proc, web);

        Ok(Manager {
            sender: b_tx,
        })
    }

    #[instrument]
    async fn process_commands(
        mut receiver: broadcast::Receiver<Command>,
        subscriptions: SubscriptionList,
        _services: ServiceList,
        clients: ClientList,
        kiln: Sender<KilnEvent>,
    ) -> Result<()> {
        while let Ok(command) = receiver.recv().await {
            match command {
                Command::Subscribe { channel, id } => {
                    trace!("subscribing to channel {}", channel);
                    let mut locked = clients.lock().unwrap();
                    let sender = locked.get_mut(&id);

                    if let Some(s) = sender {
                        let mut locked = subscriptions.lock().unwrap();
                        
                        if locked.contains_key(&channel) {
                            let subs = locked.get_mut(&channel).unwrap();
                            subs.push((id, s.clone()));
                            let _ = s.send(Message::Update { channel: "system".to_string(), data: "success".to_string() });
                        } else {
                            trace!("attempting to subscribe to a channel that doesn't exist");
                        }                    
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
                Command::ClientRegister { id, sender } => {
                    tracing::info!("registering client");
                    let mut locked = clients.lock().unwrap();

                    if locked.contains_key(&id) {
                        error!("attempting to register a client twice")
                    } else {
                        locked.insert(id, sender);
                    }
                },
                Command::Update { channel, data } => {
                    let mut locked = subscriptions.lock().unwrap();
                    trace!("updating the [{}] channel with new data", channel);

                    if locked.contains_key(&channel) {
                        locked.get_mut(&channel).unwrap().retain(|(id, sender)| {
                            trace!("updating the user [id {}] with new data", id);
                            sender.send(Message::Update {
                                channel: channel.clone(),
                                data: data.clone(),
                            }).is_ok()
                        });
                    } else {
                        error!("attempting to update a channel that doesn't exist");
                    }
                },
                Command::Ping => Manager::handle_ping(&clients),
                Command::Unknown { input } => Manager::handle_unknown(Some(input)),
                Command::StartSchedule { schedule } => {
                    let _ = kiln.send(KilnEvent::Start(schedule.normalize())).await;
                },
                Command::StopSchedule => {
                    let _ = kiln.send(KilnEvent::Stop).await;
                },
                _ => Manager::handle_unknown(None)
            }
        }

        Ok(())
    }

    #[instrument]
    fn handle_ping(clients: &ClientList) {
        trace!("handling ping");
        let name = "ping";
        clients
            .lock()
            .expect("unable to get lock on client list")
            .retain(|_id, sender| {
                sender.send(Message::Update {
                    channel: name.to_string(),
                    data: name.to_string(),
                }).is_ok()
            })
    }

    #[instrument]
    fn handle_unknown(input: Option<String>) {
        trace!("handling unknown message");
        match input {
            Some(s) => error!("unknown command received {}. ignoring", s),
            None => error!("ignoring command"),
        }        
    }
}
