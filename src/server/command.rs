use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Debug)]
pub enum Message {
    UserId(Uuid),
    Update(String),
}

/// Internal Api
#[derive(Debug, Clone)]
pub enum Command {
    Failure,

    Subscribe {
        channel: String,
        id: Uuid,
        sender: UnboundedSender<Message>,
    },

    Unsubscribe {
        channel: String,
        id: Uuid
    },

    Register {
        channel: String,
    },

    Update {
        channel: String,
        data: String,
    },

    Unknown {
        input: String,
    },
}
