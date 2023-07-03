use log;
use stream_kit::Opt;

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

    log::error!("Hello, world!");
    log::warn!("Hello, world!");
    log::info!("Hello, world!");

    Ok(())
}
