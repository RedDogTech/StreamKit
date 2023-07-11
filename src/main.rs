use stream_kit::Opt;

use srt_rs::{log as srt_log};

fn setup_logging(opt: &Opt) -> anyhow::Result<()> {
    let mut log_builder = env_logger::Builder::new();
    log_builder.parse_filters(&opt.log_level.to_string());
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (opt, _) = Opt::try_build()?;

    setup_logging(&opt)?;
    setup_srt(&opt)?;

    Ok(())
}
