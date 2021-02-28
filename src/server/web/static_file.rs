use warp::{
    filters::BoxedFilter,
    Filter,
    Reply,
};

pub fn routes() -> BoxedFilter<(impl Reply,)> {
    let public = warp::path("public").and(warp::fs::dir("public"));
    let app = warp::path("app").and(warp::filters::fs::file("public/index.html"));
    let index = warp::path::end().and(warp::filters::fs::file("public/index.html"));
    let js = warp::path("bundle.js").and(warp::filters::fs::file("public/bundle.js"));
    let build_info = warp::path("build-info").and(warp::filters::fs::file("public/build-info.json"));

    public
        .or(app)
        .or(index)
        .or(js)
        .or(build_info)
        .boxed()
}
