use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

/// 标识符校验失败的原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierError {
    /// 标识符为空。
    Empty,
    /// 标识符首尾包含空白。
    SurroundingWhitespace,
    /// 标识符包含不允许的字符。
    InvalidCharacter(char),
    /// 资源标识符是绝对路径。
    AbsolutePath,
    /// 资源标识符包含路径穿越或当前目录片段。
    PathTraversal,
    /// 资源标识符包含空的路径片段。
    EmptyPathSegment,
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("标识符不能为空"),
            Self::SurroundingWhitespace => formatter.write_str("标识符首尾不能包含空白"),
            Self::InvalidCharacter(character) => {
                write!(formatter, "标识符包含不允许的字符：{character:?}")
            }
            Self::AbsolutePath => formatter.write_str("资源标识符不能是绝对路径"),
            Self::PathTraversal => formatter.write_str("资源标识符不能包含路径穿越片段"),
            Self::EmptyPathSegment => formatter.write_str("资源标识符不能包含空的路径片段"),
        }
    }
}

impl std::error::Error for IdentifierError {}

fn is_token_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
}

fn validate(value: &str, is_path: bool) -> Result<(), IdentifierError> {
    if value.is_empty() {
        return Err(IdentifierError::Empty);
    }
    if value.trim() != value {
        return Err(IdentifierError::SurroundingWhitespace);
    }

    if !is_path {
        if let Some(character) = value
            .chars()
            .find(|character| !is_token_character(*character))
        {
            return Err(IdentifierError::InvalidCharacter(character));
        }
        return Ok(());
    }

    if value.starts_with('/') {
        return Err(IdentifierError::AbsolutePath);
    }
    if value.contains('\\') {
        return Err(IdentifierError::InvalidCharacter('\\'));
    }

    for segment in value.split('/') {
        if segment.is_empty() {
            return Err(IdentifierError::EmptyPathSegment);
        }
        if segment.trim() != segment {
            return Err(IdentifierError::SurroundingWhitespace);
        }
        if matches!(segment, "." | "..") {
            return Err(IdentifierError::PathTraversal);
        }
        if let Some(character) = segment
            .chars()
            .find(|character| !is_token_character(*character) && *character != ' ')
        {
            return Err(IdentifierError::InvalidCharacter(character));
        }
    }
    Ok(())
}

macro_rules! define_id {
    ($name:ident, $is_path:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// 创建并校验一个标识符。
            pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
                let value = value.into();
                validate(&value, $is_path)?;
                Ok(Self(value))
            }

            /// 返回标识符的规范字符串视图。
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// 取出标识符的底层字符串。
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = IdentifierError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = IdentifierError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<String> for $name {
            type Error = IdentifierError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

define_id!(AssetId, true, "版本化参考资源的相对路径标识符。");
define_id!(PlantId, false, "植物定义的稳定标识符。");
define_id!(ZombieId, false, "僵尸定义的稳定标识符。");
define_id!(LevelId, false, "关卡定义的稳定标识符。");
define_id!(ReferenceBaselineId, false, "原版行为参考基线的稳定标识符。");
