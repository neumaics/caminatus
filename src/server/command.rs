use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;
use warp::Error;
use warp::ws::Message;

use crate::schedule::Schedule;

/// External Api
#[derive(Debug, Serialize, Deserialize)]
pub enum Api {
    Schedules,
    Start {
        schedule_name: String
    },
    Subscribe {
        channel: String,
    },

    Unsubscribe {
        channel: String,
    },

    Unknown {
        input: String,
    }
}

/// Internal Api
#[derive(Debug)]
pub enum Command {
    Subscribe {
        channel: String,
        id: Uuid,
        sender: UnboundedSender<Result<Message, Error>>,
    },

    Unsubscribe {
        channel: String,
        id: Uuid
    },

    Register {
        channel: String,
    },

    Start {
        schedule: Schedule,
        simulate: bool
    },

    Stop,

    Update {
        channel: String,
        data: String,

    },
    Unknown {
        input: String,
    },
}
impl From<&str> for Api {
    fn from(string: &str) -> Self {
        match serde_json::from_str(string) {
            Ok(api) => api,
            Err(_error) => {
                Api::Unknown {
                    input: string.to_string(),
                }
            }
        }
    }
}