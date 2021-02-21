use std::time::Duration;

use anyhow::Result;
use futures::{Stream, StreamExt};
use serde_json;
use tokio::sync::mpsc;
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::sse::Event;
use warp::{filters::BoxedFilter, Filter, Reply, http::{Response, StatusCode}};

use crate::server::{Message, Command};
use crate::config::Config;
use crate::schedule::{Schedule, ScheduleError};

pub mod error;
mod schedules;

pub struct Web { }

impl Web {
    pub async fn start(conf: Config, manager_sender: Sender<Command>) -> Result<Self> {
        let m1 = manager_sender.clone();
        let m2 = manager_sender.clone();
        let m3 = manager_sender.clone();
        let manager1 = warp::any().map(move || m1.clone());
        let manager2 = warp::any().map(move || m2.clone());

        let _ = tokio::spawn(async move {
            let public = warp::path("public")
                .and(warp::fs::dir("public"));

            let app = warp::path("app")
                .and(warp::filters::fs::file("public/index.html"));

            let index = warp::path::end()
                .and(warp::filters::fs::file("public/index.html"));

            let js = warp::path("bundle.js")
                .and(warp::filters::fs::file("public/bundle.js"));
            
            let connect = warp::path("connect")
                .and(warp::get())
                .and(manager1)
                .map(|manager: Sender<Command>| {
                    let stream = warp::sse::keep_alive()
                        .interval(Duration::from_secs(5))
                        .text("bump".to_string())
                        .stream(on_connect(&manager));
                    warp::sse::reply(stream)
                });

            let subscribe = warp::path("subscribe")
                .and(warp::post())
                .and(manager2)
                .and(warp::path::param())
                .and(warp::path::param())
                .map(|manager: Sender<Command>, id: String, channel: String| {
                    let _ = manager.send(Command::Subscribe { channel: channel, id: Uuid::parse_str(&id).unwrap() });
                    Response::builder()
                        .status(warp::http::StatusCode::OK)
                        .body("{}")
                });

            let routes = index
                .or(js)
                .or(public)
                .or(app)
                .or(connect)
                .or(subscribe)
                .or(Web::device_routes(Some("./schedules".to_string()), m3))
                .or(schedules::routes(Some("./schedules".to_string())))
                .or(Web::step_routes());

            warp::serve(routes)
                .run((conf.web.host_ip, conf.web.port))
                .await;
        });

        Ok(Web { })
    }

    pub fn step_routes() -> BoxedFilter<(impl Reply,)> {
        let step = warp::get()
            .and(warp::path("step"))
            .and(warp::path("parse"))
            .and(warp::path::param())
            .map(|to_parse: String| {
                let input = percent_encoding::percent_decode(to_parse.as_bytes()).decode_utf8();

                match input {
                    Ok(i) => {
                        let parsed = Schedule::parse(&i.to_string());

                        let (code, body) = match parsed {
                            Ok(step) => (StatusCode::OK, serde_json::to_string(&step).unwrap()),
                            Err(e) => (StatusCode::NOT_ACCEPTABLE, format!(r#"{{ "message": "{:?}" }}"#, e)),
                        };
        
                        Response::builder()
                            .status(code)
                            .body(body)
                    },
                    Err(e) => {
                        let body = format!(r#"{{ "message": "error parsing request", "error": "{:?}""#, e);
                        Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(body)
                    }
                }
            });

        step.boxed()
    }    

    pub fn device_routes(directory: Option<String>, manager: Sender<Command>) -> BoxedFilter<(impl Reply,)> {
        let dir = directory.unwrap_or("./schedules".to_string());
        let dir = warp::any().map(move || dir.clone());
        let m2 = manager.clone();
        let m3 = manager.clone();
        let manager2 = warp::any().map(move || m2.clone());
        let manager3 = warp::any().map(move || m3.clone());
        
        let start = warp::get()
            .and(dir.clone())
            .and(manager2)
            .and(warp::path("device"))
            .and(warp::path("kiln"))
            .and(warp::path::param())
            .and(warp::path("start"))
            .map(|directory: String, m: Sender<Command>, id: String| match Schedule::by_name(&id, &directory) {
                Ok(s) => {
                    let normalized = s.normalize();

                    match normalized {
                        Ok(schedule) => {
                            m.clone().send(Command::StartSchedule { schedule }).expect("unable to send command to manager");
                            Response::builder()
                                .status(warp::http::StatusCode::OK)
                                .body("{ \"message\": \"started\" }".to_string())
                        }
                        Err(e) => {
                            Response::builder()
                                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                                // TODO: make this a struct
                                .body(format!("{{\"message\": \"error starting schedule with id: [{}]\", \"error\": \"{:?}\" }}", id, e))
                        }
                    }
                    
                },
                Err(error) => match error {
                    ScheduleError::IOError { description: _ } => Response::builder()
                        .status(warp::http::StatusCode::NOT_FOUND)
                        // TODO: make this a struct
                        .body(format!("{{\"message\": \"cannot find schedule with id [{}]\"}}", id)), 
                    _ => Response::builder()
                        .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                        .body(format!("{:?}", error)),
                }
            });

        let stop = warp::get()
            .and(manager3)
            .and(warp::path("device"))
            .and(warp::path("kiln"))
            .and(warp::path("stop"))
            .map(|m: Sender<Command>| {
                m.clone().send(Command::StopSchedule).expect("unable to send command to manager");
                Response::builder()
                    .status(warp::http::StatusCode::OK)
                    .body("{ \"message\": \"stopped\" }".to_string())

            });

        start.or(stop).boxed()
    }
}

fn on_connect(manager: &Sender<Command>) -> impl Stream<Item = Result<Event, warp::Error>> + Send + 'static {
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
