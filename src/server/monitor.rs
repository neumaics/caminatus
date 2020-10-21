use std::time::{Duration};

use tokio::sync::watch;
use tokio::time;

#[derive(Debug, Clone)]
pub struct Monitor {
    pub name: &'static str,
    pub receiver: tokio::sync::watch::Receiver<&'static str>
}

impl Monitor {
    pub fn start(interval: u32) -> Monitor {
        let name = "status";
        let (status_tx, status_rx) = watch::channel(name);
        let mut interval = time::interval(Duration::from_millis(interval as u64));

        tokio::spawn(async move {
            interval.tick().await;
            let _ = status_tx.broadcast("beep");
        });

        Monitor {
            name: name,
            receiver: status_rx
        }
    }
}
