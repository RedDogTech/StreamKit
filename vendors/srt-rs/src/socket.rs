use std::net::ToSocketAddrs;

use anyhow::{Result, bail};
use libc::sockaddr;
use os_socketaddr::{self, OsSocketAddr};
use libsrt_sys;

use super::error::SrtError;
use super::error;

pub struct SrtSocket{
    pub id: i32,
}

impl SrtSocket {
    pub fn new() -> Result<Self> {
        let result = unsafe { libsrt_sys::srt_create_socket() };
        if result == -1 {
            error::handle_result(Self { id: 0 }, result)
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
                return error::handle_result(self, result);
            }
        }
        bail!(SrtError::SockFail)
    }
}