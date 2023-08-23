use std::convert::Infallible;

use axum::{Router, routing::get, extract::{Path, State, Query}, http::{header, Response, StatusCode, Method}, body::Body, response::IntoResponse};
use bytes::Bytes;
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use futures::stream;
use crate::hls::SegmentStores;

pub fn create_app(store: SegmentStores) -> Router {
    Router::new()
        .route("/:id/playlist.m3u8", get(playlist))
        .route("/:id/segment.m4s", get(segment))
        .route("/:id/part.m4s", get(part))
        .route("/:id/init.mp4", get(init_segment))
        .layer(CorsLayer::new().allow_methods([Method::GET]))
        .with_state(store)
}

// Overide due the specific naming
// convention from the HLS spec
#[allow(non_snake_case)]
#[derive(Deserialize)]
struct LlhlsQueryParams {
    _HLS_msn: Option<usize>,
    _HLS_part: Option<usize>,
}

async fn playlist(Path(stream_name): Path<String>, Query(query): Query<LlhlsQueryParams>, State(state): State<SegmentStores>) -> impl IntoResponse {
    
    let sequence_number = query._HLS_msn;
    let partial_number = query._HLS_part;

    if sequence_number.is_none() && partial_number.is_some() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("missing segment seq number"))
            .unwrap()
    }

    if let Some(sequence_number) = sequence_number {
        let partial_number = partial_number.unwrap_or_default();

        let mut count = 0;
        loop {
            if count > 100 {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("missing segment seq number"))
                    .unwrap()
            }

            let lock = state.read().await;
            if let Some(store) = lock.get(&stream_name) {
                if store.partial(sequence_number, partial_number).is_some() {
                    break;
                }
            }

            count += 1;
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        }
    }

    let lock = state.read().await;
    match lock.get(&stream_name) {
        Some(store) => {
            let manifest = store.get_manifest_text().await.unwrap();
            Response::builder()
                .header(header::CONTENT_TYPE, "application/x-mpegURL")
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::CACHE_CONTROL, "max-age=0")
                .body(Body::from(manifest))
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

#[derive(Deserialize)]
struct Segment {
    msn: Option<usize>,
}

async fn segment(Path(stream_name): Path<String>, Query(query): Query<Segment>, State(state): State<SegmentStores>) -> impl IntoResponse {
    
    if let Some(msn) = query.msn {
        let mut lock = state.write().await;
        if let Some(store) = lock.get_mut(&stream_name) {
            let queue = store.segment(msn).await;

            if let Some(mut queue) = queue {
                let mut test: Vec<Result<Bytes, Infallible>> = vec![];

                while let Some(stream) = queue.recv().await {
                    match stream{
                        Some(data) => test.push(Ok(data)),
                        None => break,
                    }
                }

                return Response::builder()
                    .header(header::TRANSFER_ENCODING, "chunked")
                    .header("Content-Type", "video/mp4")
                    .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .header("Cache-Control", "max-age=31536000")
                    .body(Body::wrap_stream(stream::iter(test)))
                    .unwrap()
            }
        }
    }

    return Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "video/mp4")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header("Cache-Control", "max-age=0")
        .body(Body::from("missing segment seq number"))
        .unwrap()

}

#[derive(Deserialize)]
struct Partial {
    msn: usize,
    part: usize,
}


async fn part(Path(stream_name): Path<String>, Query(query): Query<Partial>, State(state): State<SegmentStores>) -> impl IntoResponse {
    let lock = state.read().await;

    if let Some(store) = lock.get(&stream_name) {
        if let Some(segment_bytes) = store.partial(query.msn, query.part) {
            return Response::builder()
                    .header("Content-Type", "video/mp4")
                    .header("Cache-Control", "max-age=31536000")
                    .body(Body::from(segment_bytes))
                    .unwrap()
        }
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
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