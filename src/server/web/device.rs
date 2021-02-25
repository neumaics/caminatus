use tokio::sync::broadcast::Sender;

use warp::{
    filters::BoxedFilter,
    http,
    http::{Response, StatusCode},
    Filter, Reply,
};

use crate::schedule::{Schedule, ScheduleError};
use crate::server::Command;

use super::error::ErrorResponse;

pub fn routes(directory: Option<String>, manager: Sender<Command>) -> BoxedFilter<(impl Reply,)> {
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
        .map(start);

    let stop = warp::get()
        .and(manager3)
        .and(warp::path("device"))
        .and(warp::path("kiln"))
        .and(warp::path("stop"))
        .map(stop);

    start.or(stop).boxed()
}

fn start(
    directory: String,
    manager: Sender<Command>,
    name: String,
) -> Result<Response<String>, http::Error> {
    match Schedule::by_name(&name, &directory) {
        Ok(s) => {
            let normalized = s.normalize();

            match normalized {
                Ok(schedule) => {
                    manager
                        .clone()
                        .send(Command::StartSchedule { schedule })
                        .expect("unable to send command to manager");

                    Response::builder()
                        .status(StatusCode::OK)
                        .body(r#"{ "message": "started" }"#.to_string())
                }
                Err(error) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(
                        ErrorResponse {
                            message: format!("error starting schedule with name: [{}]", &name),
                            error: format!("{:?}", error),
                        }
                        .to_string(),
                    ),
            }
        }
        Err(error) => match error {
            ScheduleError::IOError { description } => {
                Response::builder().status(StatusCode::NOT_FOUND).body(
                    ErrorResponse {
                        message: format!("unable to find schedule with name [{}]", &name),
                        error: description,
                    }
                    .to_string(),
                )
            }
            _ => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(
                    ErrorResponse {
                        message: format!("unknown error starting schedule with name [{}]", &name),
                        error: format!("{:?}", error),
                    }
                    .to_string(),
                ),
        },
    }
}

fn stop(manager: Sender<Command>) -> Result<Response<String>, http::Error> {
    manager
        .clone()
        .send(Command::StopSchedule)
        .expect("unable to send command to manager");

    Response::builder()
        .status(StatusCode::OK)
        .body(r#"{ "message": "stopped" }"#.to_string())
}
