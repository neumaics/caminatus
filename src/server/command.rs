use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Subscribe {
        channel: String,
    },

    Unsubscribe {
        channel: String,
    },

    Unknown {
        input: String,
    },
}

// FIXME: Use error in command
impl From<String> for Command {
    fn from(string: String) -> Self {
        let command: Command = match serde_json::from_str(string.as_str()) {
            Ok(command) => command,
            Err(_error) => {
                Command::Unknown { input: string }
            }
        };

        command
    }
}

// FIXME: Use error in command
impl From<&str> for Command {
    fn from(string: &str) -> Self {
        let command: Command = match serde_json::from_str(string) {
            Ok(command) => command,
            Err(_error) => {
                Command::Unknown { input: string.to_string() }
            }
        };

        command
    }
}
