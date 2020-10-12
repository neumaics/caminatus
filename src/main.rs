use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use tracing::{event, span, Level};
use tracing_subscriber;

mod config;
use config::Config;

mod oven_control;

struct Websocket;

impl Actor for Websocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Websocket {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                event!(Level::INFO, "ping");
                ctx.pong(&msg)
            },
            Ok(ws::Message::Text(text)) => {
                event!(Level::INFO, "recieved message");
                ctx.text(text)
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(Websocket {}, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let conf = Config::init().unwrap(); // TODO: Remove unwrap

    let span = span!(Level::INFO, "my_span");
    let _guard = span.enter();

    event!(Level::TRACE, process = oven_control::fuzzy().to_string().as_str());
    event!(Level::TRACE, process = oven_control::proportional().to_string().as_str());

    HttpServer::new(|| App::new().route("/status", web::get().to(index)))
        .bind(format!("{host}:{port}", host=conf.web.host, port=conf.web.port))?
        .run()
        .await
}
