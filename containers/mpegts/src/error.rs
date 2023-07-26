use thiserror::Error;

#[derive(Error, Debug)]
pub enum DemuxError {
    #[error("invalid sync_byte (expected {expected:?}, found {found:?})")]
    InvalidSyncByte {
        expected: u8,
        found: u8,
    },
}