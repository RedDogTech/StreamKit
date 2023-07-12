use anyhow::{Result};
use libc::c_int;
use libsrt_sys;

use super::socket::SrtSocket;
use super::error;

pub struct Epoll {
    id: i32,
    num_sock: usize,
}

impl Epoll {
    pub fn new() -> Result<Self> {
        let result = unsafe { libsrt_sys::srt_epoll_create() };
        if result == -1 {
            error::handle_result(Self { id: 0, num_sock: 0 }, result).map_err(anyhow::Error::from)
        } else {
            Ok(Self {
                id: result,
                num_sock: 0,
            })
        }
    }

    pub fn add(&mut self, socket: &SrtSocket, event: &libsrt_sys::SRT_EPOLL_OPT) -> Result<()> {
        let result = unsafe {
            libsrt_sys::srt_epoll_add_usock(
                self.id,
                socket.id,
                event as *const libsrt_sys::SRT_EPOLL_OPT as *const i32,
            )
        };
        if result == 0 {
            self.num_sock += 1;
        }
        error::handle_result((), result).map_err(anyhow::Error::from)
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, socket: &SrtSocket) -> Result<()> {
        let result = unsafe { libsrt_sys::srt_epoll_remove_usock(self.id, socket.id) };
        if result == 0 {
            self.num_sock -= 1;
        }
        error::handle_result((), result).map_err(anyhow::Error::from)
    }

    #[allow(dead_code)]
    pub fn update(&self, socket: &SrtSocket, event: &libsrt_sys::SRT_EPOLL_OPT) -> Result<()> {
        let result = unsafe {
            libsrt_sys::srt_epoll_update_usock(
                self.id,
                socket.id,
                event as *const libsrt_sys::SRT_EPOLL_OPT as *const i32,
            )
        };
        error::handle_result((), result).map_err(anyhow::Error::from)
    }

    #[allow(dead_code)]
    pub fn wait(&self, timeout: i64) -> Result<Vec<(SrtSocket, libsrt_sys::SRT_EPOLL_OPT)>> {
        let mut array = vec![libsrt_sys::SRT_EPOLL_EVENT { fd: 0, events: libsrt_sys::SRT_EPOLL_OPT::SRT_EPOLL_OPT_NONE }; self.num_sock];
        let result = unsafe {
            libsrt_sys::srt_epoll_uwait(
                self.id,
                array[..].as_mut_ptr() as *mut libsrt_sys::SRT_EPOLL_EVENT,
                array.len() as c_int,
                timeout,
            )
        };
        if result == -1 {
            error::handle_result(Vec::new(), result).map_err(anyhow::Error::from)
        } else {
            array.truncate(result as usize);
            Ok(array
                .iter()
                .map(|event| {
                    (
                        SrtSocket { id: event.fd },
                        event.events,
                    )
                })
                .collect())
        }
    }

    #[allow(dead_code)]
    fn clear(&mut self) -> Result<()> {
        let result = unsafe { libsrt_sys::srt_epoll_clear_usocks(self.id) };
        if result == 0 {
            self.num_sock = 0;
        }
        error::handle_result((), result).map_err(anyhow::Error::from)
    }
}

impl Drop for Epoll {
    fn drop(&mut self) {
        unsafe {
            libsrt_sys::srt_epoll_release(self.id);
        }
    }
}
