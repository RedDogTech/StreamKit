
use std::sync::Arc;

use axum::{Router, routing::get, response::{IntoResponse, Html, AppendHeaders}, extract::{Path, State}, http::{header, Response, StatusCode}, body::Body};
use tokio::sync::{Mutex, RwLock};
use crate::hls::SegmentStores;

pub fn create_app(store: SegmentStores) -> Router {
    Router::new()
        .route("/:id/playlist.m3u8", get(playlist))
        .route("/:id/segment", get(segment))
        .route("/:id/part", get(part))
        .route("/:id/init.mp4", get(init_segment))
        .with_state(store)
}

async fn playlist(Path(stream_name): Path<String>, State(state): State<SegmentStores>) -> impl IntoResponse {
    
    let lock = state.read().await;
    match lock.get(&stream_name) {
        Some(store) => {
            Response::builder()
                .header(header::CONTENT_TYPE, "application/x-mpegURL")
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::CACHE_CONTROL, "max-age=0")
                .body(Body::from("Yes!"))
                .unwrap()
        },
        None => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap()
        },
    }    
}


async fn segment(Path(stream_name): Path<String>, State(state): State<SegmentStores>) -> impl IntoResponse {
    Html("Hello, World!")
}

async fn part(Path(stream_name): Path<String>, State(state): State<SegmentStores>) -> impl IntoResponse {
    Html("Hello, World!")
}

async fn init_segment(Path(stream_name): Path<String>, State(state): State<SegmentStores>) -> impl IntoResponse {
    let lock = state.read().await;

    if let Some(store) = lock.get(&stream_name) {
        if let Some(init_bytes) = store.init_segment_ready() {
            return Response::builder()
                    .header("Content-Type", "video/mp4")
                    .header("Cache-Control", "max-age=31536000")
                    .body(Body::from(init_bytes))
                    .unwrap()
        }
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()

}