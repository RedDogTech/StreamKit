use anyhow::{bail, Result};
use std::{collections::HashMap, sync::Arc};
use crate::hls::m3u8::M3u8;

#[derive(Clone)]
pub struct SessionManager {
    stores: HashMap<String, Arc<M3u8>>,
}

impl SessionManager {
    pub fn new() -> SessionManager {
        SessionManager {
            stores: HashMap::new(),
        }
    }

    pub async fn new_store(&mut self, stream_id: &str) -> Result<()> {
        if self.check_streamid(&stream_id) {
            bail!("Stream id already exists") 
        }

        let manifest = Arc::new(M3u8::new());

        self.stores.insert(stream_id.to_string(), manifest);
        Ok(())
    }

    pub async fn get_manifest(&self, stream_id: &str) -> Option<String> {
        if let Some(data) = self.stores.get(stream_id) {
            return data.clone().get_manifest().await.ok();
        }

        None
    }

    pub fn check_streamid(&self, stream_id: &str) -> bool {
        self.stores.contains_key(stream_id)
    }

    pub fn remove_store(&mut self, stream_id: &str) -> Result<()> {
        if let Some(data) = self.stores.remove(stream_id) {
            drop(data);
        }

        Ok(())
    }

}
