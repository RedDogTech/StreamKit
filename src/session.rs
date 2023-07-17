use anyhow::{bail, Result};
use std::collections::HashMap;

use crate::store::Store;


#[derive(Clone)]
pub struct SessionManager {
    stores: HashMap<String, Store>
}

impl SessionManager {
    pub fn new() -> SessionManager {
        SessionManager {
            stores: HashMap::new(),
        }
    }

    pub fn new_store(&mut self, stream_id: String) -> Result<Store> {
        if self.check_streamid(&stream_id) {
            bail!("Stream id already exists") 
        }

        let store = Store::new();
        // let store = self.stores.insert(stream_id, store).unwrap();
        Ok(store)
    }

    pub fn check_streamid(&self, stream_id: &String) -> bool {
        self.stores.contains_key(stream_id)
    }

    pub fn get_store(&mut self, stream_id: String) -> Option<&mut Store> {
        self.stores.get_mut(&stream_id)
    }
}