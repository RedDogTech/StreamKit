use anyhow::{bail, Result};
use tokio::sync::RwLock;
use std::{collections::HashMap, sync::Arc};

use crate::hls::m3u8::M3u8;



#[derive(Clone)]
pub struct SessionManager {
    stores: Arc<RwLock<HashMap<String, Arc<M3u8>>>>,
}

impl SessionManager {
    pub fn new() -> SessionManager {
        SessionManager {
            stores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn new_store(&mut self, stream_id: &str) -> Result<()> {
        if self.check_streamid(&stream_id) {
            bail!("Stream id already exists") 
        }

        let manifest = Arc::new(M3u8::new());

        self.stores.write().await.insert(stream_id.to_string(), manifest);
        Ok(())
    }

    pub async fn get_manifest(&self, stream_id: &str) -> Option<String> {
        let cache = self.stores.read().await;

        if let Some(data) = cache.get(stream_id) {
            return data.clone().get_manifest().await.ok();
        }

        None
    }

    pub fn check_streamid(&self, stream_id: &str) -> bool {
        if let Ok(store) = self.stores.try_read() {
            return store.contains_key(stream_id);
        }
        false
    }

    // pub fn get_store(&mut self, stream_id: String) -> Option<&mut M3u8> {
    //     self.stores.get_mut(&stream_id)
    // }
}