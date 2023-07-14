use axum::{Router, routing::get, response::{IntoResponse, Html, AppendHeaders}, extract::Path, http::{header, StatusCode}};

pub fn create_app() -> Router {
    Router::new()
        .route("/:id/playlist.m3u8", get(playlist))
        .route("/:id/segment", get(segment))
        .route("/:id/part", get(part))
        .route("/:id/init", get(init))
}

async fn playlist(
    Path(_id): Path<String>,
) -> impl IntoResponse {
    

    let headers = AppendHeaders([
        (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
        (header::CACHE_CONTROL, "max-age=0"),
        (header::CONTENT_TYPE, "application/x-mpegURL")
    ]);
    
    (headers, StatusCode::NOT_FOUND)

    //Html("Hello, World!")
}


async fn segment(
    Path(_id): Path<String>,
) -> impl IntoResponse {
    Html("Hello, World!")
}

async fn part(
    Path(_id): Path<String>,
) -> impl IntoResponse {
    Html("Hello, World!")
}

async fn init(
    Path(id_id): Path<String>,
) -> impl IntoResponse {
    Html("Hello, World!")
}