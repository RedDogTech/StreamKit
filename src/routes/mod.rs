
use std::sync::Arc;

use axum::{Router, routing::get, response::{IntoResponse, Html}, extract::{Path, State}, http::{header, Response, StatusCode}, body::Body};
use tokio::sync::{Mutex, RwLock};

pub fn create_app() -> Router {
    Router::new()
        .route("/:id/playlist.m3u8", get(playlist))
        .route("/:id/segment", get(segment))
        .route("/:id/part", get(part))
        .route("/:id/init", get(init))
}

async fn playlist(Path(id): Path<String>) -> impl IntoResponse {
    
    // let headers = AppendHeaders([
    //     (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
    //     (header::CACHE_CONTROL, "max-age=0"),
    //     (header::CONTENT_TYPE, "application/x-mpegURL")
    // ]);
    
    // if let Some(text) = state.read().await.get_manifest(&id.to_owned()).await {
    //     return Response::builder()
    //         .header(header::CONTENT_TYPE, "application/x-mpegURL")
    //         .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
    //         .header(header::CACHE_CONTROL, "max-age=0")
    //         .body(Body::from(text))
    //         .unwrap()
    // }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}


async fn segment(Path(_id): Path<String>) -> impl IntoResponse {
    Html("Hello, World!")
}

async fn part(Path(_id): Path<String>) -> impl IntoResponse {
    Html("Hello, World!")
}

async fn init(Path(_id): Path<String>) -> impl IntoResponse {
    Html("Hello, World!")
}