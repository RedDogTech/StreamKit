use std::time::Duration;
use anyhow::Result;
use mpegts::{demuxer::Demuxer, DemuxerEvent, stream_type::StreamType};
use srt_rs::stream::SrtStream;
use tokio::{time::timeout, sync::oneshot};
use crate::session::Message;

use super::{ManagerHandle, Handle, ChannelMessage, Packet, Codec};

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
    demuxer: Demuxer,
    state: State,
}

impl Connection{
    pub fn new(id: u64, stream: SrtStream, manager_handle: ManagerHandle) -> Self {

        let app_name = stream.get_stream_id().ok();

        Self {
            id,
            stream,
            manager_handle,
            app_name,
            demuxer: Demuxer::new(),
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
                            for event in self.demuxer.push(&mut buf[..size])? {
                                self.handle_event(event).await?;
                            }
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

    async fn handle_event(&mut self, event: DemuxerEvent) -> Result<()> {
        match event {
            DemuxerEvent::StreamDetails(streams) => {
                let (request, response) = oneshot::channel();
                let app_name = self.app_name.clone().unwrap();

                self.manager_handle
                    .send(ChannelMessage::Create((app_name, request)))?;
                let session_sender = response.await?;

                self.state = State::Publishing(session_sender);

                log::info!("Stream Info: {:?}", &streams);
            },

            DemuxerEvent::Video(stream_type, data, pts, dts) => {
                if let State::Publishing(session) = &mut self.state {

                    if stream_type == StreamType::H264 {
                        let packet = Packet {
                            codec: Codec::H264,
                            data,
                            pts: pts.unwrap(),
                            dts,
                        };

                        session.send(Message::Packet(packet))?;
                    } else if stream_type == StreamType::H265 {
                        let packet = Packet {
                            codec: Codec::H265,
                            data,
                            pts: pts.unwrap(),
                            dts,
                        };

                        session.send(Message::Packet(packet))?;
                    }
                    
                }
            },
            DemuxerEvent::Audio(stream_type, data, pts) => {
                if let State::Publishing(session) = &mut self.state {

                    let packet = Packet {
                        codec: Codec::AAC,
                        data,
                        pts: pts.unwrap(),
                        dts: None,
                    };

                    session.send(Message::Packet(packet))?;
                }
            },
            DemuxerEvent::ClockRef(dcr) => {
                if let State::Publishing(session) = &mut self.state {
                    session.send(Message::ClockRef(dcr))?;
                }
            },
        }

        Ok(())
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