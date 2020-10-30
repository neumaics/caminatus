/// Example worker/actor service that replies "ping" on the "ping" channel.
use std::time::{Duration};

use tokio::{join, time};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::server::Command;

#[derive(Debug, Clone)]
pub struct Monitor {
    pub name: &'static str,
}

impl Monitor {
    pub async fn start(interval: u32, channel: Sender<Command>) -> Result<Monitor, String> {
        let name = "ping";
        let _ = channel.clone().send(Command::Register { channel: name.to_string() });

        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(interval as u64));
            loop {
                interval.tick().await;
                let _ = channel.send(Command::Update { channel: name.to_string(), data: name.to_string() });
            }
        });

        join!(handle);

        Ok(Monitor {
            name: name
        })
    }
}
