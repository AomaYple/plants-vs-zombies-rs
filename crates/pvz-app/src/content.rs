use pvz_core::InvalidDayLevel;
use std::{error::Error, fmt};

pub(crate) const DEFAULT_DAY_JSON: &str =
    include_str!("../../../content/levels/development-day.json");
pub(crate) const FAILURE_DAY_JSON: &str =
    include_str!("../../../content/levels/development-failure.json");
pub(crate) const MAX_DAY_CONTENT_BYTES: usize = 1_048_576;

/// 白天内容在输入大小、JSON 解析或纯规则装配边界失败。
#[derive(Debug)]
pub enum DayContentError {
    /// 解析前按 UTF-8 字节数拒绝超出一 MiB 的输入。
    TooLarge { bytes: usize },
    /// JSON 格式或纯数据 schema 无效。
    Parse(serde_json::Error),
    /// 核心拒绝规则版本、频率或规则参数。
    InvalidLevel(InvalidDayLevel),
}

impl fmt::Display for DayContentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { bytes } => write!(
                formatter,
                "白天内容超过 {MAX_DAY_CONTENT_BYTES} 字节上限，实际为 {bytes} 字节"
            ),
            Self::Parse(error) => write!(formatter, "白天内容 JSON 解析失败：{error}"),
            Self::InvalidLevel(error) => write!(formatter, "白天内容装配失败：{error}"),
        }
    }
}

impl Error for DayContentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TooLarge { .. } => None,
            Self::Parse(error) => Some(error),
            Self::InvalidLevel(error) => Some(error),
        }
    }
}
