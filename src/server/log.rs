use std::io::Result;

use tokio::sync::broadcast;
use crate::server::Command;

pub struct StreamWriter {
    pub sender: broadcast::Sender<Command>,
}

impl StreamWriter {
    pub fn init(self) -> Self {
        let _ = self.sender.send(Command::Register { channel: "log".to_string() });
        self
    }
}

impl std::io::Write for StreamWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let log = std::str::from_utf8(buf).unwrap();
        print!("{}", log);
        let _ = self.sender.send(Command::Update {
            channel: "log".to_string(),
            data: log.to_string(),
        });
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
