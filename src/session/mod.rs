use bytes::Bytes;
use tokio::sync::{mpsc, oneshot, broadcast};
pub mod manager;
pub mod connection;

#[derive(Clone, Debug)]
pub enum Codec {
    H264,
    H265,
    AAC,
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub codec: Codec,
    pub data: Bytes,
    pub pts: u64,
    pub dts: Option<u64>
}

pub enum ChannelMessage {
    Create((StreamName, Responder<Handle>)),
    Release(StreamName),
    Join((StreamName, Responder<(Handle, Watcher)>)),
    RegisterTrigger(Event, Trigger),
}

#[derive(Clone, Debug)]
pub enum Message {
    ClockRef(DCR),
    Packet(Packet),
    Disconnect,
}

pub fn trigger_channel() -> (Trigger, TriggerHandle) {
    mpsc::unbounded_channel()
}

type Event = &'static str;
type StreamName = String;
type DCR = u64;

pub type ChannelReceiver = mpsc::UnboundedReceiver<ChannelMessage>;
pub type IncomingBroadcast = mpsc::UnboundedReceiver<Message>;
pub type OutgoingBroadcast = broadcast::Sender<Message>;
pub type Handle = mpsc::UnboundedSender<Message>;
pub type Responder<P> = oneshot::Sender<P>;
pub type ManagerHandle = mpsc::UnboundedSender<ChannelMessage>;
pub type Watcher = broadcast::Receiver<Message>;
pub type Trigger = mpsc::UnboundedSender<(String, Watcher)>;
pub type TriggerHandle = mpsc::UnboundedReceiver<(String, Watcher)>;
