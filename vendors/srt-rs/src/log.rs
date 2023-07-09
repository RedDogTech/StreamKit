
pub mod log {
    use std::ffi;
    use libc::{c_char as char, c_int as int, c_void as void};
    use std::ptr;

    pub enum Level {
        Crit,
        Err,
        Warning,
        Notice,
        Info,
        Debug,
    }

    impl Level {
        fn as_cint(&self) -> int {
            match self {
                Level::Crit => 2,
                Level::Err => 3,
                Level::Warning => 4,
                Level::Notice => 5,
                Level::Info => 6,
                Level::Debug => 7,
            }
        }
    }

    pub fn set_level(level: Level) {
        unsafe {
            libsrt_sys::srt_setloglevel(level.as_cint());

            extern fn callback(_: *mut void, level: int, _: *const char, _: int, _: *const char, message: *const char) {
                
                unsafe {
                    let msg = ffi::CString::new(ffi::CStr::from_ptr(message).to_bytes()).unwrap();
                    match level {
                        7 => log::debug!("{:?}", msg),
                        6 => log::info!("{:?}", msg),
                        4 => log::warn!("{:?}", msg),
                        3 => log::error!("{:?}", msg),
                        _ => log::info!("{:?}", msg),
                    }
                }

            }

            libsrt_sys::srt_setloghandler(ptr::null_mut(), Some(callback));
        };
    }
}
