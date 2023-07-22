use std::sync::Arc;

use bytes::{BytesMut, BufMut};
use stream_kit::{Opt, session::SessionManager};
use srt_rs::log as srt_log;
use log::LevelFilter;
use stream_kit::routes;
use tokio::sync::Mutex;

fn setup_logging(opt: &Opt) -> anyhow::Result<()> {
    let mut log_builder = env_logger::Builder::new();
    log_builder.parse_filters(&opt.log_level.to_string());

    // FIXME: move this to conifg option
    log_builder.filter(Some("srt_rs::log::log"), LevelFilter::Error);
    log_builder.filter(Some("hyper::proto"), LevelFilter::Error);
    log_builder.init();
    Ok(())
}

fn setup_srt(_: &Opt) -> anyhow::Result<()> {
    let version = srt_rs::version();
    log::info!("Using srt Version: {}.{}.{}", version.0, version.1, version.2);

    srt_rs::startup()?;
    srt_log::log::set_level(srt_log::log::Level::Debug);
    Ok(())
}

async fn run_http(store: Arc<Mutex<SessionManager>>) -> anyhow::Result<()> {
    let app = routes::create_app(store);
    log::info!("starting HLS server at 127.0.0.1:3000");

    tokio::task::spawn(async move {
        // run it with hyper on localhost:3000
        axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap()
            
    }).await.map_err(anyhow::Error::from)
}

async fn run_srt(store: Arc<Mutex<SessionManager>>) -> anyhow::Result<()> {
    let test = srt_rs::builder().listen("127.0.0.1:9000", 1)?;

    log::info!("waiting for connection...");
    log::debug!("srt server running srt://127.0.0.1:9000?streamid=1234");
 
        while let Ok((peer, peer_addr)) = test.accept().await {
            log::debug!("new connection from {:?}", peer_addr);

            if let Ok(stream_id) = peer.get_stream_id() {
                if stream_id.is_empty() {
                    log::warn!("empty stream id dropping {}", peer_addr);
                    peer.close().expect("Failed to close");
                    break;
                }

                log::debug!("accepted {:?}", stream_id);
                store.lock().await.new_store(&stream_id).await?;

                let clone_store = store.clone();

                tokio::task::spawn(async move {
                    let mut buf = [0; 1316];

                    while let Ok((size, _)) = peer.recvmsg2(&mut buf).await {
                        let mut buffer = BytesMut::with_capacity(size);
                        buffer.put(&buf[..]);
                        //demux.push(&mut ctx, &buf);
                        
                        println!("");
                        println!("{:?}", buffer);
                        println!("");
                    }

                    log::info!("closing socket");
                    peer.close().expect("Failed to close");
                    clone_store.lock().await.remove_store(&stream_id).expect("Failed to close");
                });
            }
        }
    Ok(())         
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (opt, _) = Opt::try_build()?;

    setup_logging(&opt)?;
    setup_srt(&opt)?;

    let store = Arc::new(Mutex::new(SessionManager::new()));

    let _ = tokio::join!(run_http(store.clone()), run_srt(store.clone()));

    Ok(())
}
