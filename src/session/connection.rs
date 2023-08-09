use std::time::Duration;
use anyhow::Result;
use srt_rs::stream::SrtStream;
use tokio::time::timeout;
use crate::session::Message;

use super::{ManagerHandle, Handle, ChannelMessage};

const TIME_OUT: std::time::Duration = Duration::from_secs(5);

enum State {
    Initializing,
    Publishing(Handle),
    Disconnecting,
}

pub struct Connection {
    id: u64,
    manager_handle: ManagerHandle,
    app_name: Option<String>,
    stream: SrtStream,
    state: State,
}

impl Connection{
    pub fn new(id: u64, stream: SrtStream, manager_handle: ManagerHandle) -> Self {
        Self {
            id,
            stream,
            manager_handle,
            app_name: None,
            state: State::Initializing,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            match &mut self.state {
                State::Initializing | State::Publishing(_) => {
                    let mut buf = [0; 1316];
                    let message = self.stream.recvmsg2(&mut buf);

                    match timeout(TIME_OUT, message).await? {
                        Ok((size, _)) => {
                            //log::info!("GOT Bytes: {}", size);
                        }
                        _ => self.disconnect()?,
        
                    }
                }
                State::Disconnecting => {
                    log::debug!("Disconnecting...");
                    return Ok(());
                }
            }
        }
    }

    fn disconnect(&mut self) -> Result<()> {
        if let State::Publishing(session) = &mut self.state {
            let app_name = self.app_name.clone().unwrap();

            session.send(Message::Disconnect)?;

            self.manager_handle.send(ChannelMessage::Release(app_name))?;
        }

        self.state = State::Disconnecting;
        Ok(())
    }

}

impl Drop for Connection {
    fn drop(&mut self) {
        log::info!("Client {} disconnected", self.id);
    }
}