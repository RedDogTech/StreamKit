use std::{future::Future, task::{Context, Poll}, pin::Pin, thread, io::{self, Read}};
use futures::io::AsyncRead;
use anyhow::Result;
use crate::{socket::{SrtSocket, RecvMsgCtrl}, epoll::Epoll, error::SrtError};

pub struct SrtStream {
    pub socket: SrtSocket,
}

impl SrtStream {
    pub fn close(self) -> Result<()> {
        self.socket.close()
    }

    pub fn get_stream_id(&self) -> Result<String> {
        self.socket.get_stream_id()
    }

    pub fn recvmsg2<T: AsMut<[u8]>>(&self, buf: T) -> RecvMsg2<T> {
        RecvMsg2 {
            state: Some(RecvMsg2Inner {
                socket: self.socket,
                buf,
            })
        }
    }
}

impl Read for SrtStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(self.socket.recv(buf)?)
    }
}

impl Drop for SrtStream {
    fn drop(&mut self) {
        if let Err(_) = self.socket.close() {}
    }
}

pub struct RecvMsg2<T> {
    state: Option<RecvMsg2Inner<T>>,
}
struct RecvMsg2Inner<T> {
    socket: SrtSocket,
    buf: T,
}

impl<T> Future for RecvMsg2<T>
    where
        T: AsMut<[u8]> + std::marker::Unpin,
{
    type Output = Result<(usize, RecvMsgCtrl)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let ref mut inner =
            self.get_mut().state.as_mut().expect("RecvMsg2 polled after completion");
        match inner.socket.recvmsg2(inner.buf.as_mut()) {
            Ok((size, msg_ctrl)) => Poll::Ready(Ok((size, msg_ctrl))),
            Err(e) => match e {
                SrtError::AsyncRcv => {
                    let waker = cx.waker().clone();
                    let mut epoll = Epoll::new()?;
                    epoll.add(&inner.socket, &libsrt_sys::SRT_EPOLL_OPT::SRT_EPOLL_IN)?;
                    thread::spawn(move || {
                        if let Ok(_) = epoll.wait(-1) {
                            waker.wake();
                        }
                    });
                    Poll::Pending
                }
                e => Poll::Ready(Err(e.into())),
            },
        }
    }
}

impl AsyncRead for SrtStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::result::Result<usize, io::Error>> {
        match self.socket.recv(buf) {
            Ok(s) => Poll::Ready(Ok(s)),
            Err(e) => match e {
                SrtError::AsyncRcv => {
                    let waker = cx.waker().clone();
                    let mut epoll = Epoll::new().unwrap();
                    epoll.add(&self.socket, &libsrt_sys::SRT_EPOLL_OPT::SRT_EPOLL_IN).unwrap();
                    thread::spawn(move || {
                        if let Ok(_) = epoll.wait(-1) {
                            waker.wake();
                        }
                    });
                    Poll::Pending
                }
                e => Poll::Ready(Err(e.into())),
            },
        }
    }
}