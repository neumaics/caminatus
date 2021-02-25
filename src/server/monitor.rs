/// Pings the manager every so often, which will remove disconnected clients
use std::time::Duration;

use anyhow::Result;
use tokio::sync::broadcast::Sender;
use tokio::{join, time};
use tracing::{debug, trace};

use crate::server::Command;

#[derive(Debug, Clone)]
pub struct Monitor;

impl Monitor {
    pub async fn start(interval: u32, manager: Sender<Command>) -> Result<Monitor> {
        debug!("starting monitor");
        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(interval as u64));
            loop {
                trace!("pinging");
                interval.tick().await;
                let _ = manager.send(Command::Ping);
            }
        });

        let _ = join!(handle);

        Ok(Monitor)
    }
}
