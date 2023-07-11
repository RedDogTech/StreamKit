use anyhow::Result;
use libsrt_sys;

pub mod log;
pub mod epoll;
pub mod socket;
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
        error::handle_result((), result)
    }
}

pub fn cleanup() -> Result<()> {
    let result = unsafe { libsrt_sys::srt_cleanup() };
    error::handle_result((), result)
}
