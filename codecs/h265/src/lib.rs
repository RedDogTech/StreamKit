use std::fmt::{Debug, self};
use annexb::AnnexB;
use common::{FormatReader, FormatWriter, ReadFormat, WriteFormat};
use config::HEVCDecoderConfigurationRecord;
use error::HevcError;
use hvcc::Hvcc;

pub mod annexb;
pub mod hvcc;
pub mod config;
//pub mod sps;
mod error;
pub mod nal;

pub struct Hevc(Vec<nal::Unit>);

impl From<Vec<nal::Unit>> for Hevc {
    fn from(val: Vec<nal::Unit>) -> Self {
        Self(val)
    }
}

impl From<Hevc> for Vec<nal::Unit> {
    fn from(val: Hevc) -> Self {
        val.0
    }
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Initializing,
    Ready,
}

impl Default for State {
    fn default() -> Self {
        Self::Initializing
    }
}

#[derive(Default)]
pub struct H265Coder {
    pub dcr: Option<HEVCDecoderConfigurationRecord>,
    state: State,
}

impl H265Coder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_dcr<D>(&mut self, dcr: D) -> Result<(), HevcError>
    where
        D: TryInto<HEVCDecoderConfigurationRecord, Error = HevcError>,
    {
        let dcr = dcr.try_into()?;
        self.dcr = Some(dcr);
        self.state = State::Ready;
        Ok(())
    }
}

impl Debug for H265Coder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HevcDecoder")
            .field("state", &self.state)
            .finish()
    }
}

impl FormatReader<Hvcc> for H265Coder {
    type Output = Hevc;
    type Error = HevcError;

    fn read_format(
        &mut self,
        format: Hvcc,
        input: &[u8],
    ) -> Result<Option<Self::Output>, Self::Error> {
        Ok(match &self.state {
            State::Initializing => {
                self.set_dcr(input)
                    .map_err(|_| HevcError::DecoderInitializationFailed)?;
                None
            }
            State::Ready => {
                let mut dcr = self.dcr.as_mut().unwrap();
                Some(format.read_format(input, &mut dcr)?)
            }
        })
    }
}

impl FormatReader<AnnexB> for H265Coder {
    type Output = Hevc;
    type Error = HevcError;

    fn read_format(
        &mut self,
        format: AnnexB,
        input: &[u8],
    ) -> Result<Option<Self::Output>, Self::Error> {
        Ok(match &self.state {
            State::Initializing => {
                self.dcr = Some(HEVCDecoderConfigurationRecord::default());
                let mut dcr = self.dcr.as_mut().unwrap();
                let nals = format.read_format(input, &mut dcr)?;
                self.state = State::Ready;
                Some(nals)
            }
            State::Ready => {
                let mut dcr = self.dcr.as_mut().unwrap();
                Some(format.read_format(input, &mut dcr)?)
            }
        })
    }
}

impl FormatWriter<AnnexB> for H265Coder {
    type Input = Hevc;
    type Error = HevcError;

    fn write_format(&mut self, format: AnnexB, input: Self::Input) -> Result<Vec<u8>, Self::Error> {
        match &self.state {
            State::Initializing => Err(HevcError::NotInitialized),
            State::Ready => {
                let dcr = self.dcr.as_ref().unwrap();
                Ok(format.write_format(input, dcr)?)
            }
        }
    }
}