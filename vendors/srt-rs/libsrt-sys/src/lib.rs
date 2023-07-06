#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

pub use libc::{c_char as char, c_int as int, c_void as void, sockaddr, sockaddr_storage, socklen_t};

pub type SRTSOCKET = int;

#[link(name = "srt")]
extern "C" {
    ///
    /// Get SRT version value
    /// 
    pub fn srt_getversion()  -> int;

    ///
    /// Called at the start of an application that uses the SRT library
    /// 
    pub fn srt_startup() -> int;

    ///
    /// Cleans up global SRT resources before exiting an application
    /// 
    pub fn srt_cleanup() -> int;

    ///
    /// Creates an SRT socket.
    /// 
    pub fn srt_create_socket() -> SRTSOCKET;

    ///
    /// Closes the socket or group and frees all used resources.
    /// Note that underlying UDP sockets may be shared between sockets,
    /// so these are freed only with the last user closed.
    /// 
    pub fn srt_close(srt_socket: SRTSOCKET) -> int;
}