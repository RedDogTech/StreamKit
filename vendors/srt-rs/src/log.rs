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

        fn from_cint(level: int) -> Level {
            match level {
                2 => Level::Crit,
                3 => Level::Err,
                4 => Level::Warning,
                5 => Level::Notice ,
                6 => Level::Info,
                7 => Level::Debug,
                _ => unreachable!()
            }
        }
    }

    pub fn set_level(level: Level) {
        unsafe {
            libsrt_sys::srt_setloglevel(level.as_cint());

            extern fn callback(_: *mut void, level: int, _: *const char, _: int, _: *const char, message: *const char) {
                
                unsafe {
                    let msg = ffi::CString::new(ffi::CStr::from_ptr(message).to_bytes()).unwrap();
                    match Level::from_cint(level) {
                        Level::Debug => log::debug!("{:?}", msg),
                        Level::Info  => log::info!("{:?}", msg),
                        Level::Warning | Level::Notice => log::warn!("{:?}", msg),
                        Level::Err | Level::Crit => log::error!("{:?}", msg),
                    }
                }

            }

            libsrt_sys::srt_setloghandler(ptr::null_mut(), Some(callback));
        };
    }
}
