use std::time::Duration;

use anyhow::Result;
use futures::{Stream, StreamExt};
use serde_json;
use tokio::sync::mpsc;
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::sse::Event;
use warp::{filters::BoxedFilter, Filter, Reply, http::Response};

use crate::server::{Message, Command};
use crate::config::Config;
use crate::schedule::{Schedule, ScheduleError};

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
                .or(Web::schedule_routes(Some("./schedules".to_string())));

            warp::serve(routes)
                .run((conf.web.host_ip, conf.web.port))
                .await;
        });

        Ok(Web { })
    }

    pub fn schedule_routes(directory: Option<String>) -> BoxedFilter<(impl Reply,)> {
        let dir = directory.unwrap_or("./schedules".to_string());
        let dir = warp::any().map(move || dir.clone());

        let schedules = warp::get()
            .and(warp::path("schedules"))
            .and(warp::path::end())
            .and(dir.clone())
            .map(|directory: String| serde_json::to_string(&Schedule::all(&directory)).unwrap());
        
        let schedule = warp::get()
            .and(warp::path("schedules"))
            .and(warp::path::param())
            .and(dir.clone())
            .map(|id: String, directory: String| match Schedule::by_name(&id, &directory) {
                Ok(s) => Response::builder()
                    .status(warp::http::StatusCode::OK)
                    .body(serde_json::to_string(&s).unwrap()),
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

        let new_schedule = warp::post()
            .and(warp::path("schedules"))
            .and(warp::path::end())
            .and(warp::body::content_length_limit(1024 * 32))
            .and(warp::body::json())
            .and(dir.clone())
            .map(|body: Schedule, directory: String| {
                match Schedule::new(body, &directory) {
                    Ok(s) => Response::builder()
                        .status(warp::http::StatusCode::OK)
                        .body(serde_json::to_string(&s).unwrap()),
                    Err(err) => Response::builder()
                        .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                        .body(format!("{:?}", err)),
                }
                
            });

        let update_schedule = warp::put()
            .and(warp::path("schedules"))
            .and(warp::path::param())
            .and(warp::path::end())
            .and(warp::body::content_length_limit(1024 * 32))
            .and(warp::body::json())
            .and(dir.clone())
            .map(|schedule_name: String, body: Schedule, directory: String| {
                Schedule::update(schedule_name, body, &directory).unwrap()
            });
        
        let delete_schedule = warp::delete()
            .and(warp::path("schedules"))
            .and(warp::path::param())
            .and(dir.clone())
            .and(warp::path::end())
            .map(|schedule_id: String, directory: String| Schedule::delete(schedule_id, &directory).unwrap());

        schedules
            .or(schedule)
            .or(new_schedule)
            .or(update_schedule)
            .or(delete_schedule)
            .boxed()
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

#[cfg(test)]
mod route_tests {
    use tempfile::tempdir;
    use std::fs;
    use anyhow::Result;
    use super::*;

    #[tokio::test]
    async fn should_get_all_available_schedules() {
        let filter = Web::schedule_routes(Some("./tests/sample_schedules".to_string()));
    
        let response = warp::test::request()
            .path("/schedules")
            .reply(&filter)
            .await;
    
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn should_get_schedule_by_id() {
        let filter = Web::schedule_routes(Some("./tests/sample_schedules".to_string()));
    
        let response = warp::test::request()
            .path("/schedules/valid")
            .reply(&filter)
            .await;
    
        assert_eq!(response.status(), 200);

        let response = warp::test::request()
            .path("/schedules/definitely_doesnt_exist")
            .reply(&filter)
            .await;
    
        assert_eq!(response.status(), 404);
    }

    #[tokio::test]
    async fn should_create_new_schedule() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("");
        let valid = fs::read_to_string("./tests/sample_schedules/valid.json")?;
        let filter = Web::schedule_routes(Some(file_path.into_os_string().into_string().unwrap()));
        let response = warp::test::request()
            .method("POST")
            .path("/schedules")
            .body(&valid)
            .reply(&filter)
            .await;

        dir.close()?;
        assert_eq!(response.status(), 200);

        Ok(())
    }

    #[tokio::test]
    async fn should_delete_schedule() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("");
        let valid = fs::read_to_string("./tests/sample_schedules/valid.json")?;
        let filter = Web::schedule_routes(Some(file_path.clone().into_os_string().into_string().unwrap()));
        let response = warp::test::request()
            .method("POST")
            .path("/schedules")
            .body(&valid)
            .reply(&filter)
            .await;
        
        assert_eq!(response.status(), 200);

        let new_file = response.body();
        let file_string = &String::from_utf8(new_file.to_vec()).unwrap().replace('\"', "");
        let response = warp::test::request()
            .method("DELETE")
            .path(format!("/schedules/{}", file_string).as_str())
            .reply(&filter)
            .await;
   
        dir.close()?;
        assert_eq!(response.status(), 200);

        Ok(())
    }
}
