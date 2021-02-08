use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Debug)]
pub enum Message {
    UserId(Uuid),
    Update {
        channel: String,
        data: String
    },
}

/// Internal Api
#[derive(Debug, Clone)]
pub enum Command {
    Failure,

    Subscribe {
        channel: String,
        id: Uuid,
    },

    Unsubscribe {
        channel: String,
        id: Uuid,
    },

    Register {
        channel: String,
    },

    ClientRegister {
        id: Uuid,
        sender: UnboundedSender<Message>,
    },

    Update {
        channel: String,
        data: String,
    },

    Unknown {
        input: String,
    },

    Ping,
}
