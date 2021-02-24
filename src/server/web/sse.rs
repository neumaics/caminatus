use std::time::Duration;

use anyhow::Result;
use futures::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::sse::Event;
use warp::{filters::BoxedFilter, Filter, Reply, http, http::{Response, StatusCode}};

use crate::server::{Message, Command};

pub fn routes(manager_sender: &Sender<Command>) -> BoxedFilter<(impl Reply,)> {
    let manager1 = manager_sender.clone();
    let manager2 = manager_sender.clone();

    let connect = warp::path("connect")
        .and(warp::get())
        .and(warp::any().map(move || manager1.clone()))
        .map(connect);

    let sub = warp::path("subscribe")
        .and(warp::post())
        .and(warp::any().map(move || manager2.clone()))
        .and(warp::path::param())
        .and(warp::path::param())
        .map(subscribe);

    connect
        .or(sub)
        .boxed()
}

fn connect(manager: Sender<Command>) -> impl Reply {
    let stream = warp::sse::keep_alive()
        .interval(Duration::from_secs(5))
        .text("bump".to_string())
        .stream(on_connect(manager));
    
    warp::sse::reply(stream)
}

fn on_connect(manager: Sender<Command>) -> impl Stream<Item = Result<Event, warp::Error>> + Send + 'static {
    let id = Uuid::new_v4();
    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);
    
    tx.send(Message::UserId(id)).unwrap();

    let _ = manager.send(Command::ClientRegister { id, sender: tx });

    rx.map(|msg| match msg {
        Message::UserId(id) => Ok(Event::default().event("id").data(id.to_string())),
        Message::Update { channel, data } => Ok(Event::default().event(channel).data(data)),
    })
}

fn subscribe(manager: Sender<Command>, id: String, channel: String) -> Result<Response<String>, http::Error> {
    let _ = manager.send(Command::Subscribe { channel: channel, id: Uuid::parse_str(&id).unwrap() });
    Response::builder()
        .status(warp::http::StatusCode::OK)
        .body("{}".to_string())
}
