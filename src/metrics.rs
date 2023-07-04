use lazy_static::lazy_static;
use prometheus::{
    opts, register_int_gauge_vec, IntGaugeVec,
};

lazy_static! {

    pub static ref STREAMKIT_SRT_STREAMS_IN: IntGaugeVec = register_int_gauge_vec!(
        opts!("STREAMKIT_SRT_STREAMS_IN", "StreamKit stream in count"),
        &["index"]
    )
    .expect("Can't create a metric");

}