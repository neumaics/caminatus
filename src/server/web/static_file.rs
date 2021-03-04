use rust_embed::RustEmbed;
use warp::{
    filters::BoxedFilter,
    http::{Error, HeaderValue},
    path::Tail,
    reply::Response,
    Filter, Reply,
};

#[derive(RustEmbed)]
#[folder = "public"]
struct Asset;

pub fn routes() -> BoxedFilter<(impl Reply,)> {
    let public = warp::path("public").and(warp::path::tail()).map(serve_path);
    let app = warp::path("app").map(|| serve_file("index.html"));
    let index = warp::path::end().map(|| serve_file("index.html"));
    let js = warp::path("bundle.js").map(|| serve_file("bundle.js"));
    let build_info = warp::path("build-info").map(|| serve_file("build-info.json"));

    app.or(js).or(build_info).or(index).or(public).boxed()
}

fn serve_path(path: Tail) -> Result<impl Reply, Error> {
    serve_file(path.as_str())
}

fn serve_file(path: &str) -> Result<impl Reply, Error> {
    let asset = Asset::get(path)
        .ok_or_else(warp::reject::not_found)
        .unwrap();
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = Response::new(asset.into());
    res.headers_mut().insert(
        "content_type",
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}
