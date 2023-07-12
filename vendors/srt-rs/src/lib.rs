use std::{net::{ToSocketAddrs, SocketAddr}, future::Future, task::{Context, Poll}, pin::Pin, thread};

use anyhow::Result;
use epoll::Epoll;
use error::SrtError;
use libsrt_sys;
use socket::SrtSocket;
use stream::SrtStream;

pub mod log;
pub mod epoll;
pub mod socket;
pub mod stream;
pub mod error;

pub fn version() -> (i32, i32, i32) {
    let version = unsafe { 
        libsrt_sys::srt_getversion()
    };

    let major = version / 0x10000;
    let minor = (version / 0x100) % 0x100;
    let patch = version % 0x100;

    (major, minor, patch)
}

pub fn startup() -> Result<()> {
    let result = unsafe { libsrt_sys::srt_startup() };
    if result == 1 {
        Ok(())
    } else {
        error::handle_result((), result).map_err(anyhow::Error::from)
    }
}

pub fn cleanup() -> Result<()> {
    let result = unsafe { libsrt_sys::srt_cleanup() };
    error::handle_result((), result).map_err(anyhow::Error::from)
}

pub fn builder() -> SrtBuilder {
    SrtBuilder {
    }
}

pub struct SrtBuilder {

}

impl SrtBuilder {
    pub fn listen<A: ToSocketAddrs>(self, addr: A, backlog: i32) -> Result<SrtListener> {
        let socket = SrtSocket::new()?;
        let socket = socket.bind(addr)?;
        socket.listen(backlog)?; // Still synchronous
        Ok(SrtListener { socket })
    }
}

pub struct SrtListener {
    socket: SrtSocket,
}

impl SrtListener {
    pub fn accept(&self) -> AcceptFuture {
        AcceptFuture {
            socket: self.socket,
        }
    }
    pub fn close(self) -> Result<()> {
        self.socket.close()
    }
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
}

impl Drop for SrtListener {
    fn drop(&mut self) {
        if let Err(_) = self.socket.close() {}
    }
}

pub struct AcceptFuture {
    socket: SrtSocket,
}

impl Future for AcceptFuture {
    type Output = Result<(SrtStream, SocketAddr)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.socket.accept() {
            Ok((socket, addr)) => {
                let r_b = socket.set_receive_blocking(false);
                let s_b = socket.set_send_blocking(false);
                if r_b.is_err() {
                    Poll::Ready(Err(r_b.expect_err("unreachable")))
                } else if s_b.is_err() {
                    Poll::Ready(Err(s_b.expect_err("unreachable")))
                } else {
                    Poll::Ready(Ok((SrtStream { socket }, addr)))
                }
            }
            Err(e) => match e {
                SrtError::AsyncRcv => {
                    let waker = cx.waker().clone();
                    let mut epoll = Epoll::new()?;
                    epoll.add(&self.socket, &libsrt_sys::SRT_EPOLL_OPT::SRT_EPOLL_IN)?;
                    thread::spawn(move || {
                        if let Ok(_) = epoll.wait(-1) {
                            waker.wake();
                        }
                    });
                    Poll::Pending
                }
                e => {
                    Poll::Ready(Err(e.into()))
                },
            },
        }
    }
}

