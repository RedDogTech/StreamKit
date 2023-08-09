use tokio::sync::{mpsc, oneshot, broadcast};
pub mod manager;
pub mod connection;

#[derive(Clone)]
pub struct Packet {

}

pub enum ChannelMessage {
    Create((String, Responder<Handle>)),
    Release(String),
    Join((String, Responder<(Handle, Watcher)>)),
    RegisterTrigger(String, Trigger),
}

pub enum Message {
    Packet(),
    InitData(
        Responder<()>,
    ),
    Disconnect,
}

pub type ChannelReceiver = mpsc::UnboundedReceiver<ChannelMessage>;
pub type IncomingBroadcast = mpsc::UnboundedReceiver<Message>;
pub type OutgoingBroadcast = broadcast::Sender<Packet>;
pub type Handle = mpsc::UnboundedSender<Message>;
pub type Trigger = mpsc::UnboundedSender<(String, Watcher)>;
pub type Responder<P> = oneshot::Sender<P>;
pub type ManagerHandle = mpsc::UnboundedSender<ChannelMessage>;
pub type Watcher = broadcast::Receiver<Packet>;
