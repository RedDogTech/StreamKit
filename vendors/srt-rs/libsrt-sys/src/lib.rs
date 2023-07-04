#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

pub use libc::{c_char as char, c_int as int, c_void as void, sockaddr, sockaddr_storage, socklen_t};

#[link(name = "srt")]
extern "C" {
    pub fn srt_getversion()  -> int;
}