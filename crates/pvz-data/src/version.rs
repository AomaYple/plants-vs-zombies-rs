use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

/// 当前支持的数据 schema 版本。
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion(u16);

/// 尝试构造不支持的 schema 版本时返回的错误。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct UnsupportedSchemaVersion(pub u16);

impl fmt::Display for UnsupportedSchemaVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "不支持的数据 schema 版本：{}", self.0)
    }
}

impl std::error::Error for UnsupportedSchemaVersion {}

impl SchemaVersion {
    /// 当前 schema 版本。
    pub const CURRENT: Self = Self(1);

    /// 创建受支持的 schema 版本。
    pub const fn new(value: u16) -> Result<Self, UnsupportedSchemaVersion> {
        if value == Self::CURRENT.0 {
            Ok(Self(value))
        } else {
            Err(UnsupportedSchemaVersion(value))
        }
    }

    /// 返回当前 schema 版本。
    pub const fn current() -> Self {
        Self::CURRENT
    }

    /// 返回数值形式的 schema 版本。
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl Serialize for SchemaVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(self.0)
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u16::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// 游戏规则数据的主次版本。
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RulesVersion {
    major: u16,
    minor: u16,
}

/// 规则版本值或文本格式无效。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InvalidRulesVersion {
    /// 主版本必须大于零。
    ZeroMajor,
    /// 文本不是严格的 `主版本.次版本` 格式。
    InvalidFormat,
    /// 版本数字超出 `u16` 范围。
    OutOfRange,
}

impl fmt::Display for InvalidRulesVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroMajor => formatter.write_str("规则版本主版本必须大于零"),
            Self::InvalidFormat => formatter.write_str("规则版本必须使用主版本.次版本格式"),
            Self::OutOfRange => formatter.write_str("规则版本数字超出范围"),
        }
    }
}

impl std::error::Error for InvalidRulesVersion {}

impl RulesVersion {
    /// 当前开发中的规则版本。
    pub const CURRENT: Self = Self { major: 2, minor: 0 };

    /// 创建规则版本。
    pub const fn new(major: u16, minor: u16) -> Result<Self, InvalidRulesVersion> {
        if major == 0 {
            Err(InvalidRulesVersion::ZeroMajor)
        } else {
            Ok(Self { major, minor })
        }
    }

    /// 当前规则版本。
    pub const fn current() -> Self {
        Self::CURRENT
    }

    /// 返回主版本。
    pub const fn major(self) -> u16 {
        self.major
    }

    /// 返回次版本。
    pub const fn minor(self) -> u16 {
        self.minor
    }
}

impl fmt::Display for RulesVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for RulesVersion {
    type Err = InvalidRulesVersion;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (major, minor) = value
            .split_once('.')
            .ok_or(InvalidRulesVersion::InvalidFormat)?;
        if major.is_empty()
            || minor.is_empty()
            || major.contains('.')
            || minor.contains('.')
            || (major.len() > 1 && major.starts_with('0'))
            || (minor.len() > 1 && minor.starts_with('0'))
            || !major.chars().all(|character| character.is_ascii_digit())
            || !minor.chars().all(|character| character.is_ascii_digit())
        {
            return Err(InvalidRulesVersion::InvalidFormat);
        }

        let major = major
            .parse::<u16>()
            .map_err(|_| InvalidRulesVersion::OutOfRange)?;
        let minor = minor
            .parse::<u16>()
            .map_err(|_| InvalidRulesVersion::OutOfRange)?;
        Self::new(major, minor)
    }
}

impl Serialize for RulesVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RulesVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}
