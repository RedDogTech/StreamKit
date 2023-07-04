use anyhow::{Result, bail};
use libsrt_sys;


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
    let result = unsafe { 
        libsrt_sys::srt_startup()
    };

    if result > 1 {
        bail!("Failed to start srt instance")
    } 
    
    Ok(())
}

pub fn shutdown() -> Result<()> {
    let result = unsafe { 
        libsrt_sys::srt_cleanup()
    };
    
    if result > 1 {
        bail!("Failed to cleanup srt")
    } 
    
    Ok(())
}

struct SrtServer {

}

impl SrtServer {
    pub fn builder() -> SrtBuilder {
        SrtBuilder::default()
    }
}

#[derive(Default)]
struct SrtBuilder {

}

impl SrtBuilder {
    pub fn new() -> SrtBuilder {

        SrtBuilder {
            
        }
    }
}
