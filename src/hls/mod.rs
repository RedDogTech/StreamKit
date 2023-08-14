use anyhow::{Result, Ok};
use std::{sync::Arc, collections::HashMap};
use tokio::sync::RwLock;
use crate::{session::ManagerHandle, routes};
use self::segment_store::SegmentStore;

pub mod m3u8;
pub mod segment_store;
pub type SegmentStores = Arc<RwLock<HashMap<String, SegmentStore>>>;

pub struct Service {
}

impl Service {
    pub fn new() -> Self {
        Self { }
    }

    pub async fn run(self, stores: SegmentStores, port: u32)-> Result<()> {
        let app = routes::create_app(stores);
        log::info!("starting HLS server at 127.0.0.1:3000");
    
        let _ = axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
            .serve(app.into_make_service())
            .await;
        
        Ok(())
    }
}