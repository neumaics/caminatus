/// Example worker/actor service that replies "ping" on the "ping" channel.
use std::time::{Duration};

use tokio::{join, time};
use tokio::sync::broadcast::Sender;

use crate::server::Command;

const NAME: &str = "ping";

#[derive(Debug, Clone)]
pub struct Monitor {
    pub name: &'static str,
}

impl Monitor {
    pub async fn start(interval: u32, channel: Sender<Command>) -> Result<Monitor, String> {
        let name = NAME.to_string();
        let _ = channel.clone().send(Command::Register { channel: name.clone() });

        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(interval as u64));
            loop {
                interval.tick().await;
                let _ = channel.send(Command::Update { channel: name.clone(), data: name.clone() });
            }
        });

        let _ = join!(handle); // todo: use value somehow

        Ok(Monitor {
            name: NAME
        })
    }
}
