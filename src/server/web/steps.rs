use serde_json;
use warp::{filters::BoxedFilter, Filter, Reply, http, http::{Response, StatusCode}};

use crate::schedule::Schedule;

pub fn routes() -> BoxedFilter<(impl Reply,)> {
    let step = warp::get()
        .and(warp::path("step"))
        .and(warp::path("parse"))
        .and(warp::path::param())
        .map(parse);

    step.boxed()
}  

fn parse(to_parse: String) -> Result<Response<String>, http::Error> {
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
            let body = format!(r#"{{ "message": "error parsing request", "error": "{:?}" }}"#, e);
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(body)
        }
    }
}
