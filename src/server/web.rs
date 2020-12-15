use futures::{FutureExt, StreamExt};
use serde_json;
use tokio::{task, join};
use tokio::sync::mpsc;
use tokio::sync::broadcast::Sender;
use tracing::{debug, error, info};
use uuid::Uuid;
use warp::ws::{WebSocket};
use warp::{filters::BoxedFilter, Filter, Reply, http::Response};

use crate::server::{Command, Api};
use crate::config::Config;
use crate::schedule::{Schedule, ScheduleError};

pub struct Web { }

impl Web {
    pub async fn start(conf: Config, manager_sender: Sender<Command>) -> Result<Self, String> {
        let manager = warp::any().map(move || manager_sender.clone());

        let _ = tokio::spawn(async move {
            let ws = warp::path("ws")
                .and(warp::ws())
                .and(manager)
                .map(|ws: warp::ws::Ws, manager: Sender<Command>| {
                    ws.on_upgrade(move |socket| on_connect(manager, socket))
                });

            let public = warp::path("public")
                .and(warp::fs::dir("public"));

            let routes = ws
                .or(public)
                .or(Web::schedule_routes(Some("./schedules".to_string())));

            warp::serve(routes)
                .run((conf.web.host_ip, conf.web.port))
                .await;
        }).await;

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
            .map(|schedule_name: String, body: Schedule| Schedule::update(schedule_name, body).unwrap());
        
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
}

async fn on_connect(manager: Sender<Command>, ws: WebSocket) {
    let id = Uuid::new_v4();
    let (user_ws_tx, mut user_ws_rx) = ws.split();
    let copy2 = manager.clone();

    info!("client connecting");
    let (tx, rx) = mpsc::unbounded_channel();
    let forwarder = task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            error!("websocket send error: {}", e);
        }
    }));

    let cmd_tx = tx.clone();
    let command_reader = task::spawn(async move {
        let directory = "./schedules".to_string();

        while let Some(result) = user_ws_rx.next().await {
            debug!("{:?}", result);
            let message = match result {
                Ok(message) => message,
                Err(error) => {
                    debug!("{:?}", error);
                    break;
                }
            };
            
            if message.to_str().is_ok() {
                let api: Api = Api::from(message.to_str().unwrap());

                let command = match api {
                    Api::Schedules => Command::Unknown { input: "schedules".to_string() },
                    Api::Subscribe { channel } => Command::Subscribe {
                        channel: channel,
                        id: id,
                        sender: cmd_tx.clone()
                    },
                    Api::Unsubscribe { channel } => Command::Unsubscribe {
                        channel: channel,
                        id: id,
                    },
                    Api::Start { schedule_name } => Command::Forward {
                        channel: "client".to_string(),
                        cmd: Box::new(Command::Start {
                            simulate: false,
                            schedule: Schedule::by_name(&schedule_name, &directory).unwrap()
                    })},
                    Api::Unknown { input } => Command::Unknown { input: input },
                };
                let _ = copy2.send(command);
            }
        }

        let _ = on_disconnect();
    });

    join!(command_reader, forwarder);
}

async fn on_disconnect() {
    info!("client disconnecting");
}

#[cfg(test)]
mod route_tests {
    use tempfile::tempdir;
    use std::fs;
    use std::fs::File;
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