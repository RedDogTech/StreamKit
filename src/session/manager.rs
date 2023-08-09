use std::{sync::Arc, collections::HashMap};
use tokio::sync::{RwLock, mpsc, broadcast};
use super::{ManagerHandle, ChannelReceiver, Trigger, Handle, OutgoingBroadcast, ChannelMessage};
use anyhow::{Result, bail};


pub struct SessionManager {
    handle: ManagerHandle,
    incoming: ChannelReceiver,
    channels: Arc<RwLock<HashMap<String, (Handle, OutgoingBroadcast)>>>,
    triggers: Arc<RwLock<HashMap<String, Vec<Trigger>>>>,
}

impl SessionManager {

    pub fn new() -> Self {
        let (handle, incoming) = mpsc::unbounded_channel();
        let channels = Arc::new(RwLock::new(HashMap::new()));
        let triggers = Arc::new(RwLock::new(HashMap::new()));

        Self {
            handle,
            incoming,
            channels,
            triggers,
        }
    }

    pub fn handle(&self) -> ManagerHandle {
        self.handle.clone()
    }

    async fn process_message(&mut self, message: ChannelMessage) -> Result<()> {
        match message {
            ChannelMessage::Create((name, responder)) => {

                let (handle, incoming) = mpsc::unbounded_channel();
                let (outgoing, _watcher) = broadcast::channel(64);
                let mut sessions = self.channels.write().await;
                sessions.insert(name.clone(), (handle.clone(), outgoing.clone()));

                let triggers = self.triggers.read().await;
                if let Some(event_triggers) = triggers.get("create_session") {
                    for trigger in event_triggers {
                        trigger.send((name.clone(), outgoing.subscribe()))?;
                    }
                }

                tokio::spawn(async move {

                });

                if let Err(_) = responder.send(handle) {
                    bail!("Failed to send response");
                }

            },
            ChannelMessage::Release(_) => todo!(),
            ChannelMessage::Join(_) => todo!(),
            ChannelMessage::RegisterTrigger(_, _) => todo!(),
        }
        Ok(())
    }

    pub async fn run(mut self) {
        while let Some(message) = self.incoming.recv().await {
            if let Err(err) = self.process_message(message).await {
                log::error!("{}", err);
            };
        }
    }
}