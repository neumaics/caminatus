use std::time::{Duration};

use tokio::sync::mpsc::Sender;
use tokio::{join, time};
use tracing::info;

use crate::server::Command;

#[derive(Debug, Clone)]
pub struct Monitor {
    pub name: &'static str,
}

impl Monitor {
    pub async fn start(interval: u32, mut manager_sender: Sender<Command>) -> Result<Monitor, String> {
        let name = "ping";
        
        let mut bcast = manager_sender.clone();
        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(interval as u64));
            loop {
                interval.tick().await;
                let _ = bcast.send(Command::Update { channel: name.to_string() }).await;
            }
        });

        info!("registering the ping service");
        let register = manager_sender.send(Command::Register { channel: name.to_string() });

        join!(register, handle);

        Ok(Monitor {
            name: name
        })
    }
}
