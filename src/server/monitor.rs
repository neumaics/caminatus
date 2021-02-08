/// Pings the manager every so often, which will remove disconnected clients
use std::time::{Duration};

use tokio::{join, time};
use tokio::sync::broadcast::Sender;

use crate::server::Command;

#[derive(Debug, Clone)]
pub struct Monitor;

impl Monitor {
    pub async fn start(interval: u32, channel: Sender<Command>) -> Result<Monitor, String> {
        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(interval as u64));
            loop {
                interval.tick().await;
                let _ = channel.send(Command::Ping);
            }
        });

        let _ = join!(handle); // todo: use value somehow

        Ok(Monitor)
    }
}
