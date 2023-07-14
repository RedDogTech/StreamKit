use stream_kit::Opt;
use srt_rs::log as srt_log;
use log::LevelFilter;
use stream_kit::routes;

fn setup_logging(opt: &Opt) -> anyhow::Result<()> {
    let mut log_builder = env_logger::Builder::new();
    log_builder.parse_filters(&opt.log_level.to_string());

    // FIXME: move this to conifg option
    log_builder.filter(Some("srt_rs::log::log"), LevelFilter::Error);
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

async fn run_http() -> anyhow::Result<()> {
    
    let app = routes::create_app();

    log::info!("starting HLS server at 127.0.0.1:3000");

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .map_err(anyhow::Error::from)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (opt, _) = Opt::try_build()?;

    setup_logging(&opt)?;
    setup_srt(&opt)?;

    let test = srt_rs::builder().listen("127.0.0.1:9000", 1)?;

    log::info!("waiting for connection...");
    log::debug!("srt server running srt://127.0.0.1:4532?streamid=1234");

    run_http().await?;

    loop {
        let (peer, peer_addr) = test.accept().await?;

        tokio::spawn(async move {
            log::debug!("new connection from {:?}", peer_addr);

            if let Ok(streamid) = peer.get_stream_id() {
                if streamid.is_empty() {
                    log::warn!("empty stream id dropping {}", peer_addr);
                    peer.close().expect("Failed to close");
                    return;
                }
            }

            let mut buf = [0; 1316];
            while let Ok((size, _)) = peer.recvmsg2(&mut buf).await {

                //println!("got {:?}", buf.len());
                //println!("expected {:?}", size);
            }

            log::info!("closing socket");
            peer.close().expect("Failed to close");
        });
    };
}
