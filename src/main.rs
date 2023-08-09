use std::sync::Arc;
use stream_kit::{Opt, mpegts_ingest, srt::SrtService, session::manager::SessionManager};
use srt_rs::log as srt_log;
use log::LevelFilter;
use stream_kit::routes;
use tokio::{sync::{Mutex, RwLock}, task::JoinSet};
use anyhow::Result;

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

    let mut handles = Vec::new();
    let manager = SessionManager::new();
    let manager_handle = manager.handle();

    handles.push(tokio::spawn(manager.run()));
    handles.push(tokio::spawn(SrtService::new(manager_handle).run(9000)));


    //let _ = tokio::join!(manager.run(), SrtService::new(manager_handle).run(9000));

    for handle in handles {
        handle.await?;
    }
    
    Ok(())
}








// const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
// const GIT_SHA: &str = env!("GIT_SHA");



// fn setup_srt(_: &Opt) -> anyhow::Result<()> {
//     let version = srt_rs::version();
//     log::info!("Using srt Version: {}.{}.{}", version.0, version.1, version.2);

//     srt_rs::startup()?;
//     srt_log::log::set_level(srt_log::log::Level::Debug);
//     Ok(())
// }

// async fn run_http(store: Arc<RwLock<SessionManager>>) -> anyhow::Result<()> {
//     let app = routes::create_app(store);
//     log::info!("starting HLS server at 127.0.0.1:3000");

//     tokio::task::spawn(async move {
//         // run it with hyper on localhost:3000
//         axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
//             .serve(app.into_make_service())
//             .await
//             .unwrap()
            
//     }).await.map_err(anyhow::Error::from)
// }

// async fn run_srt(store: Arc<RwLock<SessionManager>>) -> anyhow::Result<()> {
//     let test = srt_rs::builder().listen("127.0.0.1:9000", 1)?;

//     log::info!("waiting for connection...");
//     log::debug!("srt server running srt://127.0.0.1:9000?streamid=1234");

//     let clone_store = Arc::clone(&store);
 
//     while let Ok((peer, peer_addr)) = test.accept().await {
//         log::debug!("new connection from {:?}", peer_addr);

//         if let Ok(stream_id) = peer.get_stream_id() {
//             if stream_id.is_empty() {
//                 log::warn!("empty stream id dropping {}", peer_addr);
//                 peer.close().expect("Failed to close");
//                 break;
//             }

//             let clone_store = Arc::clone(&clone_store);

//             log::debug!("accepted {:?}", stream_id);
//             //let clone_store = Arc::clone(&store);
//             clone_store.write().await.new_manifest(&stream_id).await?;

//             tokio::task::spawn(async move {
//                 let spwaned_store = Arc::clone(&clone_store);

//                 let mut buf = [0; 1316];
//                 let mut demux =  mpegts_ingest::create_demux();

//                 while let Ok((size, _)) = peer.recvmsg2(&mut buf).await {
//                     let _ = demux.push(&mut buf[..size]);
//                 }

//                 log::info!("closing socket");
//                 peer.close().expect("Failed to close");
//                 spwaned_store.write().await.remove_store(&stream_id).expect("Failed to close");
//             });
//         }
//     }
//     Ok(())         
// }

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let (opt, _) = Opt::try_build()?;

//     println!("{}", u64::pow(3, 22));

//     setup_logging(&opt)?;
//     log::info!("Starting StreamKit {{ \"Version\": \"{CARGO_PKG_VERSION}\", \"GitSha\": \"{GIT_SHA}\" }}");

//     setup_srt(&opt)?;

//     let store = Arc::new(RwLock::new(SessionManager::new()));

//     let _ = tokio::join!(run_http(store.clone()), run_srt(store.clone()));

//     Ok(())
// }
