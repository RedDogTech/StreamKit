use log;
use stream_kit::Opt;

use srt_rs;

fn setup(opt: &Opt) -> anyhow::Result<()> {
    let mut log_builder = env_logger::Builder::new();
    log_builder.parse_filters(&opt.log_level.to_string());
    log_builder.init();
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (opt, _) = Opt::try_build()?;

    setup(&opt)?;

    let version = srt_rs::version();

    log::info!("Using srt Version: {}.{}.{}", version.0, version.1, version.2);
    srt_rs::startup()?;

    Ok(())
}
