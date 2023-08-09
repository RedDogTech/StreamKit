use std::net::SocketAddr;
use crate::session::{ManagerHandle, connection::Connection};
use anyhow::Result;
use srt_rs::stream::SrtStream;

pub struct SrtService {
    manager_handle: ManagerHandle,
    client_id: u64,
}

impl SrtService {
    pub fn new(manager_handle: ManagerHandle) -> Self {
        srt_rs::startup().expect("Failed to start SRT libs");
        srt_rs::log::log::set_level(srt_rs::log::log::Level::Debug);

        let version = srt_rs::version();
        log::info!("Using srt Version: {}.{}.{}", version.0, version.1, version.2);
        Self {
            manager_handle,
            client_id: 0,
        }
    }

    pub async fn run(mut self, port: i32) {
        if let Err(err) = self.handle_srt(port).await {
            log::error!("{}", err);
        }
    }

    async fn handle_srt(&mut self, port: i32) -> Result<()> {
        let addr = format!("127.0.0.1:{}", port);
        let test = srt_rs::builder().listen(&addr, 1)?;
        log::info!("Listening for SRT connections on {}", addr);

        loop {
            let (peer_stream, peer_addr) = test.accept().await?;

            self.process(peer_stream, peer_addr);
            self.client_id += 1;
        }
    }

    fn process(&self, stream: SrtStream, peer: SocketAddr) {
        log::info!("New client connection: {}, ({})", &self.client_id, &peer);

        let id = self.client_id;
        let conn = Connection::new(id, stream, self.manager_handle.clone());

        tokio::spawn(async move {
            if let Err(err) = conn.run().await {
                log::error!("{}", err);
            }
        });
    }

}

