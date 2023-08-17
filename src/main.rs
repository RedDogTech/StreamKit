use std::{sync::Arc, collections::HashMap};
use lazy_static::*;
use stream_kit::{Opt, srt::SrtService, session::manager::SessionManager, fmp4, hls::{SegmentStores, self}};
use log::LevelFilter;
use anyhow::Result;
use tokio::sync::RwLock;

lazy_static! {
    static ref SESSION_STORES: SegmentStores = Arc::new(RwLock::new(HashMap::new()));
}

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_SHA: &str = env!("GIT_SHA");

fn setup_logging(opt: &Opt) -> anyhow::Result<()> {
    let mut log_builder = env_logger::Builder::new();
    log_builder.parse_filters(&opt.log_level.to_string());

    // FIXME: move this to conifg option
    log_builder.filter(Some("srt_rs::log::log"), LevelFilter::Error);
    log_builder.filter(Some("hyper::proto"), LevelFilter::Error);
    log_builder.init();
    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {
    let (opt, _) = Opt::try_build()?;

    setup_logging(&opt)?;

    log::info!("Starting StreamKit {{ \"Version\": \"{CARGO_PKG_VERSION}\", \"GitSha\": \"{GIT_SHA}\" }}");

    let mut handles = Vec::new();
    let manager = SessionManager::new();
    let manager_handle = manager.handle();

    //
    // Spawn stream manager to distribute streams
    //
    handles.push(tokio::spawn(manager.run()));

    //
    // mpegts -> fmp4 & hls (output)
    // 
    {
        let manager_handle_t = manager_handle.clone();
        handles.push(tokio::spawn(async {
            _ = fmp4::Service::new(manager_handle_t, opt).run(Arc::clone(&SESSION_STORES)).await;
         }));

         handles.push(tokio::spawn(async move {
            _ = hls::Service::new().run(Arc::clone(&SESSION_STORES), 3000).await;
        }));
    }
    
    //
    //  Handle the SRt input and deplexing
    // 
    handles.push(tokio::spawn(SrtService::new(manager_handle).run(9000)));

    for handle in handles {
        handle.await?;
    }
    
    Ok(())
}
