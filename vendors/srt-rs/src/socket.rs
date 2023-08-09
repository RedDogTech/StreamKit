use std::mem;
use std::net::{ToSocketAddrs, SocketAddr};
use std::num::NonZeroI64;

use anyhow::{Result, bail};
use libc::{sockaddr, c_int, c_void, c_char};
use os_socketaddr::{self, OsSocketAddr};
use libsrt_sys;

use super::error::SrtError;
use super::error;

#[derive(Copy, Clone, Debug)]
pub struct SrtSocket{
    pub id: i32,
}

impl SrtSocket {
    pub fn new() -> Result<Self> {
        let result = unsafe { libsrt_sys::srt_create_socket() };
        if result == -1 {
            error::handle_result(Self { id: 0 }, result).map_err(anyhow::Error::from)
        } else {
            Ok(Self { id: result })
        }
    }
    pub fn bind<A: ToSocketAddrs>(self, addrs: A) -> Result<Self> {
        if let Ok(addrs) = addrs.to_socket_addrs() {
            for addr in addrs {
                let os_addr: OsSocketAddr = addr.into();
                let result = unsafe {
                    libsrt_sys::srt_bind(
                        self.id,
                        os_addr.as_ptr() as *const sockaddr,
                        os_addr.len() as i32,
                    )
                };
                return error::handle_result(self, result).map_err(anyhow::Error::from);
            }
        }
        bail!(SrtError::SockFail)
    }

    pub fn listen(&self, backlog: i32) -> Result<()> {
        let result = unsafe { libsrt_sys::srt_listen(self.id, backlog) };
        error::handle_result((), result).map_err(anyhow::Error::from)
    }

    pub fn accept(&self) -> Result<(Self, SocketAddr), SrtError> {
        let mut addr = OsSocketAddr::new();
        let mut _addrlen: c_int = addr.capacity() as i32;
        let result = unsafe {
            libsrt_sys::srt_accept(
                self.id,
                addr.as_mut_ptr() as *mut sockaddr,
                &mut _addrlen as *mut c_int,
            )
        };
        if result == -1 {
            error::handle_result((Self { id: 0 }, "0.0.0.0:0".parse().unwrap()), result)
        } else {
            Ok((Self { id: result }, addr.into_addr().unwrap()))
        }
    }

    pub fn close(self) -> Result<()> {
        let result = unsafe { libsrt_sys::srt_close(self.id) };
        error::handle_result((), result).map_err(anyhow::Error::from)
    }

    pub fn recv(&self, buf: &mut [u8]) -> Result<usize, SrtError> {
        let result =
            unsafe { libsrt_sys::srt_recv(self.id, buf as *mut [u8] as *mut c_char, buf.len() as i32) };
        if result == -1 {
            error::handle_result(result as usize, result)
        } else {
            Ok(result as usize)
        }
    }

    pub fn recvmsg2(&self, buf: &mut [u8]) -> Result<(usize, RecvMsgCtrl), SrtError> {
        let mut msg_ctl = libsrt_sys::SRT_MSGCTRL {
            flags: 0,
            msgttl: 0,
            inorder: 0,
            boundary: 0,
            srctime: 0,
            pktseq: 0,
            msgno: 0,
            grpdata: std::ptr::null_mut() as *mut libsrt_sys::SRT_SOCKGROUPDATA,
            grpdata_size: 0,
        };
        let result =
            unsafe { libsrt_sys::srt_recvmsg2(self.id, buf as *mut [u8] as *mut c_char, buf.len() as i32, &mut msg_ctl as *mut _) };
        if result == -1 {
            Err(error::get_last_error().into())
        } else {
            Ok((
                result as usize,
                RecvMsgCtrl {
                    src_time: NonZeroI64::new(msg_ctl.srctime),
                    pkt_seq: msg_ctl.pktseq,
                    msg_no: msg_ctl.msgno,
                    _priv: ()
                }
            ))
        }
    }

}

impl SrtSocket {
    pub fn local_addr(&self) -> Result<SocketAddr> {
        let mut addr = OsSocketAddr::new();
        let mut addrlen: c_int = addr.capacity() as i32;
        let result = unsafe {
            libsrt_sys::srt_getsockname(
                self.id,
                addr.as_mut_ptr() as *mut sockaddr,
                &mut addrlen as *mut c_int,
            )
        };
        if result == -1 {
            error::handle_result("0.0.0.0:0".parse().unwrap(), result).map_err(anyhow::Error::from)
        } else {
            error::handle_result(addr.into_addr().unwrap(), 0).map_err(anyhow::Error::from)
        }
    }

    pub fn get_stream_id(&self) -> Result<String> {
        let mut id = String::from_iter([' '; 512].iter());
        let mut id_len = mem::size_of_val(&id) as i32;
        let result = unsafe {
            libsrt_sys::srt_getsockflag(
                self.id,
                libsrt_sys::SRT_SOCKOPT::SRTO_STREAMID,
                id.as_mut_ptr() as *mut c_void,
                &mut id_len as *mut c_int,
            )
        };
        id.truncate(id_len as usize);
        error::handle_result(id, result).map_err(anyhow::Error::from)
    }

    pub fn set_receive_blocking(&self, blocking: bool) -> Result<()> {
        let result = unsafe {
            libsrt_sys::srt_setsockflag(
                self.id,
                libsrt_sys::SRT_SOCKOPT::SRTO_RCVSYN,
                &blocking as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result).map_err(anyhow::Error::from)
    }

    pub fn set_send_blocking(&self, blocking: bool) -> Result<()> {
        let result = unsafe {
            libsrt_sys::srt_setsockflag(
                self.id,
                libsrt_sys::SRT_SOCKOPT::SRTO_SNDSYN,
                &blocking as *const bool as *const c_void,
                mem::size_of::<bool>() as c_int,
            )
        };
        error::handle_result((), result).map_err(anyhow::Error::from)
    }
}



pub struct RecvMsgCtrl {
    pub src_time: Option<NonZeroI64>,
    pub pkt_seq: i32,
    pub msg_no: i32,
    _priv: (),
}