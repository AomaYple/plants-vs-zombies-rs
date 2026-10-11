use crate::diagnostics::{AuditReport, Diagnostic};
use pvz_data::AssetId;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

const SUPPORTED_SCHEMA_VERSION: u16 = 1;
const EXPECTED_ROOT: &str = "assets/original";

#[derive(Debug)]
pub enum AssetError {
    Io { path: PathBuf, message: String },
    Json { message: String },
    UnsupportedSchema { version: u16 },
    InvalidRoot { root: String },
    InvalidPath { path: String, message: String },
    PathCollision { first: String, second: String },
    InvalidRecord { path: String, message: String },
    Xml { message: String },
}

impl fmt::Display for AssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => {
                write!(formatter, "读取资源输入失败：{}：{message}", path.display())
            }
            Self::Json { message } => write!(formatter, "资源清单 JSON 无效：{message}"),
            Self::UnsupportedSchema { version } => {
                write!(formatter, "不支持的资源清单 schema 版本：{version}")
            }
            Self::InvalidRoot { root } => write!(formatter, "资源清单根目录无效：{root}"),
            Self::InvalidPath { path, message } => {
                write!(formatter, "资源清单路径无效：{path}：{message}")
            }
            Self::PathCollision { first, second } => {
                write!(formatter, "资源清单路径存在大小写冲突：{first} 与 {second}")
            }
            Self::InvalidRecord { path, message } => {
                write!(formatter, "资源清单记录无效：{path}：{message}")
            }
            Self::Xml { message } => write!(formatter, "XML 资源文本无效：{message}"),
        }
    }
}

impl std::error::Error for AssetError {}

#[derive(Debug, Deserialize)]
struct RawManifest {
    schema_version: u16,
    root: String,
    files: Vec<RawRecord>,
}

#[derive(Debug, Deserialize)]
struct RawRecord {
    path: String,
    kind: String,
    size: u64,
    sha256: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum AssetKind {
    Text,
    Binary,
    Package,
}

impl AssetKind {
    fn parse(path: &str, value: &str) -> Result<Self, AssetError> {
        match value {
            "text" => Ok(Self::Text),
            "binary" => Ok(Self::Binary),
            "package" => Ok(Self::Package),
            _ => Err(AssetError::InvalidRecord {
                path: path.to_owned(),
                message: format!("不支持的资源类型：{value}"),
            }),
        }
    }
}

#[derive(Debug, Clone)]
struct AssetRecord {
    id: AssetId,
    kind: AssetKind,
    size: u64,
    sha256: String,
}

/// 仓库资源根目录的打开入口。
pub struct AssetRoot;

impl AssetRoot {
    /// 从仓库根目录读取 `assets/manifest.json` 并校验清单结构。
    pub fn open(repository_root: &Path) -> Result<AssetIndex, AssetError> {
        let manifest_path = repository_root.join("assets/manifest.json");
        let bytes = fs::read(&manifest_path).map_err(|error| io_error(manifest_path, error))?;
        let raw: RawManifest =
            serde_json::from_slice(&bytes).map_err(|error| AssetError::Json {
                message: error.to_string(),
            })?;
        if raw.schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(AssetError::UnsupportedSchema {
                version: raw.schema_version,
            });
        }
        if raw.root != EXPECTED_ROOT {
            return Err(AssetError::InvalidRoot { root: raw.root });
        }

        let mut records = Vec::with_capacity(raw.files.len());
        let mut casefolded = std::collections::BTreeMap::new();
        for item in raw.files {
            let id = AssetId::new(item.path.clone()).map_err(|error| AssetError::InvalidPath {
                path: item.path.clone(),
                message: error.to_string(),
            })?;
            if let Some(first) =
                casefolded.insert(id.as_str().to_ascii_lowercase(), item.path.clone())
            {
                return Err(AssetError::PathCollision {
                    first,
                    second: item.path,
                });
            }
            let kind = AssetKind::parse(&item.path, &item.kind)?;
            if item.sha256.len() != 64
                || !item
                    .sha256
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
            {
                return Err(AssetError::InvalidRecord {
                    path: item.path,
                    message: "sha256 必须是 64 位十六进制字符串".to_owned(),
                });
            }
            records.push(AssetRecord {
                id,
                kind,
                size: item.size,
                sha256: item.sha256,
            });
        }
        records.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        Ok(AssetIndex {
            root: repository_root.join("assets/original"),
            records,
        })
    }
}

/// 通过清单索引访问资源输入并生成只读审计报告。
#[derive(Debug)]
pub struct AssetIndex {
    root: PathBuf,
    records: Vec<AssetRecord>,
}

impl AssetIndex {
    /// 返回清单登记的资源数量。
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// 清单没有登记资源时返回 `true`。
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// 对登记文件和目录中的额外文件执行只读审计。
    pub fn audit(&self) -> AuditReport {
        let mut diagnostics = Vec::new();
        let mut expected = std::collections::BTreeSet::new();
        for record in &self.records {
            let relative = record.id.as_str().to_owned();
            expected.insert(relative.clone());
            let path = self.root.join(record.id.as_str());
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    diagnostics.push(Diagnostic::Missing { path: relative });
                    continue;
                }
                Err(error) => {
                    diagnostics.push(Diagnostic::Io {
                        path: relative,
                        message: error.to_string(),
                    });
                    continue;
                }
            };
            if bytes.len() as u64 != record.size {
                diagnostics.push(Diagnostic::SizeMismatch {
                    path: relative,
                    expected: record.size,
                    actual: bytes.len() as u64,
                });
            } else if hex_digest(&bytes) != record.sha256 {
                diagnostics.push(Diagnostic::HashMismatch { path: relative });
            }
            let _kind = record.kind;
        }
        collect_unexpected(&self.root, &self.root, &expected, &mut diagnostics);
        AuditReport::new(diagnostics)
    }
}

fn collect_unexpected(
    root: &Path,
    directory: &Path,
    expected: &std::collections::BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_unexpected(root, &path, expected, diagnostics);
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if !expected.contains(&relative) {
                diagnostics.push(Diagnostic::Unexpected { path: relative });
            }
        }
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn io_error(path: PathBuf, error: io::Error) -> AssetError {
    AssetError::Io {
        path,
        message: error.to_string(),
    }
}
