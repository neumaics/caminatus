use serde::Deserialize;
use serde_json;
use warp::{filters::BoxedFilter, Filter, Reply, http::{Response, StatusCode}};

use crate::schedule::{Schedule, ScheduleError};
use super::error::ErrorResponse;

const ROOT: &str = "schedules";
const LENGTH_LIMIT: u64 = 1024 * 32;

#[derive(Deserialize)]
struct ScheduleParams {
    pub normalize: Option<bool>
}

pub fn routes(directory: Option<String>) -> BoxedFilter<(impl Reply,)> {
    let dir = directory.unwrap_or("./schedules".to_string());
    let dir = warp::any().map(move || dir.clone());

    let schedules = warp::get()
        .and(dir.clone())
        .and(warp::path(ROOT))
        .and(warp::path::end())
        .map(list);
    
    let schedule = warp::get()
        .and(dir.clone())
        .and(warp::path(ROOT))
        .and(warp::path::param())
        .and(warp::query::<ScheduleParams>())
        .map(by_name);
    
    let new_schedule = warp::post()
        .and(dir.clone())
        .and(warp::path(ROOT))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(LENGTH_LIMIT))
        .and(warp::body::json())
        .map(new);

    let update_schedule = warp::put()
        .and(dir.clone())
        .and(warp::path(ROOT))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(LENGTH_LIMIT))
        .and(warp::body::json())
        .and(warp::path::param())
        .map(update);
    
    let delete_schedule = warp::delete()
        .and(dir.clone())
        .and(warp::path(ROOT))
        .and(warp::path::param())
        .and(warp::path::end())
        .map(delete);

    schedules

        .or(schedule)
        .or(new_schedule)
        .or(update_schedule)
        .or(delete_schedule)
        .boxed()
}

fn list(directory: String) -> Result<Response<String>, warp::http::Error> {
    Response::builder()
        .status(StatusCode::OK)
        .body(serde_json::to_string(&Schedule::all(&directory)).unwrap())
}

fn by_name(directory: String, name: String, params: ScheduleParams) -> Result<Response<String>, warp::http::Error> {
    let norm = params.normalize.unwrap_or(false);

    match Schedule::by_name(&name, &directory) {
        Ok(s) => {
            if norm {
                Response::builder()
                    .status(StatusCode::OK)
                    .body(serde_json::to_string(&s.normalize().unwrap()).unwrap())
            } else {
                Response::builder()
                    .status(StatusCode::OK)
                    .body(serde_json::to_string(&s).unwrap())
            }
        }
        Err(error) => match error {
            ScheduleError::IOError { description } => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(ErrorResponse {
                    message: format!("cannot find schedule with name [{}]", &name),
                    error: description,
                }.to_string()),
            _ => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(ErrorResponse {
                    message: "unknown error".to_string(),
                    error: format!("{:?}", error)
                }.to_string()),
        }
    }
}

fn new(directory: String, schedule: Schedule) -> Result<Response<String>, warp::http::Error> {
    match Schedule::new(schedule, &directory) {
        Ok(s) => Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string(&s).unwrap()),
        Err(err) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("{:?}", err)),
    }
}

fn update(directory: String, schedule: Schedule, name: String) -> Result<Response<String>, warp::http::Error> {
    match Schedule::update(name, schedule, &directory) {
        Ok(s) => Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string(&s).unwrap()),
        Err(err) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("{:?}", err)),
    }
}

fn delete(directory: String, name: String) -> Result<Response<String>, warp::http::Error> {
    match Schedule::delete(name, &directory) {
        Ok(s) => Response::builder()
        .status(StatusCode::OK)
        .body(s),
    Err(err) => Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(format!("{:?}", err)),
    }
}

#[cfg(test)]    
mod route_tests {
    use tempfile::tempdir;
    use std::fs;
    use anyhow::Result;
    use super::*;

    #[tokio::test]
    async fn should_get_all_available_schedules() {
        let filter = routes(Some("./tests/sample_schedules".to_string()));
    
        let response = warp::test::request()
            .path("/schedules")
            .reply(&filter)
            .await;
    
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn should_get_schedule_by_id() {
        let filter = routes(Some("./tests/sample_schedules".to_string()));
    
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
    async fn should_accept_normalize_parameter() {
        let filter = routes(Some("./tests/sample_schedules".to_string()));
    
        let response = warp::test::request()
            .path("/schedules/valid?normalize=true")
            .reply(&filter)
            .await;
    
        assert_eq!(response.status(), 200);

        let response = warp::test::request()
            .path("/schedules/valid?normalize=false")
            .reply(&filter)
            .await;
    
        assert_eq!(response.status(), 200);
    }

    
    #[tokio::test]
    async fn should_validate_normalize_parameter() {
        let filter = routes(Some("./tests/sample_schedules".to_string()));

        let response = warp::test::request()
            .path("/schedules/valid?normalize=not%20a%20boolean")
            .reply(&filter)
            .await;
    
        assert_eq!(response.status(), 400);
    }

    #[tokio::test]
    async fn should_create_new_schedule() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("");
        let valid = fs::read_to_string("./tests/sample_schedules/valid.json")?;
        let filter = routes(Some(file_path.into_os_string().into_string().unwrap()));
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
        let filter = routes(Some(file_path.clone().into_os_string().into_string().unwrap()));
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
