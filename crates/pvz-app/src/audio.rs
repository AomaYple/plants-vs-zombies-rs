use std::{fmt, io::Cursor};
/// 预解码验证摘要，播放由平台适配层负责。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioInfo {
    pub channels: u8,
    pub sample_rate: u32,
    pub samples: u64,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AudioError {
    UnsupportedMo3,
    InvalidOgg,
    EmptyOgg,
}
impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedMo3 => "MO3 音乐尚未支持：需要独立解包与模块播放器",
            Self::InvalidOgg => "OGG 音频解码失败",
            Self::EmptyOgg => "OGG 音频没有可播放样本",
        })
    }
}
impl std::error::Error for AudioError {}
pub fn validate_audio(bytes: &[u8]) -> Result<AudioInfo, AudioError> {
    if bytes.starts_with(b"MO3") {
        return Err(AudioError::UnsupportedMo3);
    }
    let mut reader = lewton::inside_ogg::OggStreamReader::new(Cursor::new(bytes))
        .map_err(|_| AudioError::InvalidOgg)?;
    let channels = reader.ident_hdr.audio_channels;
    let sample_rate = reader.ident_hdr.audio_sample_rate;
    let mut samples = 0;
    while let Some(packet) = reader
        .read_dec_packet_itl()
        .map_err(|_| AudioError::InvalidOgg)?
    {
        samples += packet.len() as u64;
    }
    if samples == 0 || channels == 0 || sample_rate == 0 {
        return Err(AudioError::EmptyOgg);
    }
    Ok(AudioInfo {
        channels,
        sample_rate,
        samples,
    })
}
