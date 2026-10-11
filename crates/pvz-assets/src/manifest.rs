use crate::diagnostics::{AuditReport, Diagnostic};
use pvz_data::{AssetId, ReferenceBaselineId};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum AssetError {
    Io {
        path: PathBuf,
        kind: io::ErrorKind,
        message: String,
    },
    Json {
        message: String,
    },
    UnsupportedSchema {
        version: u16,
    },
    InvalidRoot {
        root: String,
    },
    InvalidPath {
        path: String,
        message: String,
    },
    PathCollision {
        first: String,
        second: String,
    },
    InvalidRecord {
        path: String,
        message: String,
    },
    Xml {
        message: String,
    },
}
impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                path,
                kind,
                message,
            } => write!(
                f,
                "读取资源输入失败：{}：{kind:?}：{message}",
                path.display()
            ),
            Self::Json { message } => write!(f, "资源清单 JSON 无效：{message}"),
            Self::UnsupportedSchema { version } => {
                write!(f, "不支持的资源清单 schema 版本：{version}")
            }
            Self::InvalidRoot { .. } => write!(f, "资源清单根目录无效"),
            Self::InvalidPath { message, .. } => write!(f, "资源清单路径无效：{message}"),
            Self::PathCollision { first, second } => {
                write!(f, "资源清单路径存在大小写冲突：{first} 与 {second}")
            }
            Self::InvalidRecord { path, message } => {
                write!(f, "资源清单记录无效：{path}：{message}")
            }
            Self::Xml { message } => write!(f, "XML 资源文本无效：{message}"),
        }
    }
}
impl std::error::Error for AssetError {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    schema_version: u16,
    baseline_id: String,
    root: String,
    source_archive: SourceArchive,
    text_normalization: String,
    files: Vec<RawRecord>,
    package: PackageMetadata,
}
/// 原始资源包的来源指纹。
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceArchive {
    pub name: String,
    pub size: u64,
    pub sha256: String,
}
/// 包索引元数据；校验一致性，不解释包载荷格式。
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageMetadata {
    pub path: String,
    pub entry_count: u64,
    pub index_bytes: u64,
    pub size: u64,
    pub sha256: String,
    pub extraction_check: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRecord {
    path: String,
    kind: AssetKind,
    source_size: u64,
    source_sha256: String,
    size: u64,
    sha256: String,
    source_encoding: Option<String>,
}
/// 登记资源的格式类别。
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Text,
    Binary,
    Package,
}
/// 校验过的资源记录；相对路径可用于浏览器资源提供器。
#[derive(Debug, Clone)]
pub struct AssetRecord {
    id: AssetId,
    kind: AssetKind,
    source_size: u64,
    source_sha256: String,
    size: u64,
    sha256: String,
    source_encoding: Option<String>,
}
impl AssetRecord {
    pub fn id(&self) -> &AssetId {
        &self.id
    }
    pub const fn kind(&self) -> AssetKind {
        self.kind
    }
    pub const fn size(&self) -> u64 {
        self.size
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub const fn source_size(&self) -> u64 {
        self.source_size
    }
    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }
    pub fn source_encoding(&self) -> Option<&str> {
        self.source_encoding.as_deref()
    }
    /// 校验平台提供的字节，无文件系统副作用。
    pub fn validate_bytes(&self, bytes: &[u8]) -> AuditReport {
        let path = self.id.as_str().to_owned();
        let mut diagnostics = Vec::new();
        if bytes.len() as u64 != self.size {
            diagnostics.push(Diagnostic::SizeMismatch {
                path: path.clone(),
                expected: self.size,
                actual: bytes.len() as u64,
            });
        } else if hex_digest(bytes) != self.sha256 {
            diagnostics.push(Diagnostic::HashMismatch { path: path.clone() });
        }
        if self.kind == AssetKind::Text {
            let valid = std::str::from_utf8(bytes).is_ok()
                && !bytes.starts_with(b"\xef\xbb\xbf")
                && !bytes.contains(&b'\r')
                && (bytes.is_empty() || bytes.ends_with(b"\n"));
            if !valid {
                diagnostics.push(Diagnostic::InvalidText { path });
            }
        }
        AuditReport::new(diagnostics)
    }
}
/// 纯字节清单模型，不绑定宿主路径。
#[derive(Debug, Clone)]
pub struct Manifest {
    baseline_id: String,
    source_archive: SourceArchive,
    text_normalization: String,
    package: PackageMetadata,
    records: Vec<AssetRecord>,
}
impl Manifest {
    pub fn parse(bytes: &[u8]) -> Result<Self, AssetError> {
        let raw: RawManifest = serde_json::from_slice(bytes).map_err(|e| AssetError::Json {
            // 不转发输入中的无效字符串，避免恶意输入把绝对路径带入诊断。
            message: format!(
                "结构或字段不符合 schema（行 {}，列 {}）",
                e.line(),
                e.column()
            ),
        })?;
        if raw.schema_version != 1 {
            return Err(AssetError::UnsupportedSchema {
                version: raw.schema_version,
            });
        }
        if raw.root != "assets/original" {
            return Err(AssetError::InvalidRoot {
                root: "〈无效根目录〉".to_owned(),
            });
        }
        if ReferenceBaselineId::new(raw.baseline_id.clone()).is_err()
            || raw.text_normalization.trim().is_empty()
        {
            return Err(invalid(
                "清单",
                "baseline_id 与 text_normalization 不得为空",
            ));
        }
        let mut records = Vec::with_capacity(raw.files.len());
        let mut casefolded = std::collections::BTreeMap::new();
        for item in raw.files {
            let id = AssetId::new(item.path.clone()).map_err(|_| AssetError::InvalidPath {
                path: "〈无效路径〉".to_owned(),
                message: "必须是规范化的相对 POSIX 路径".to_owned(),
            })?;
            if let Some(first) = casefolded.insert(item.path.to_lowercase(), item.path.clone()) {
                return Err(AssetError::PathCollision {
                    first,
                    second: item.path,
                });
            }
            let suffix = Path::new(&item.path)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let classified = match item.kind {
                AssetKind::Text => {
                    ["xml", "reanim", "txt", "trail", "jsfl"].contains(&suffix.as_str())
                }
                AssetKind::Binary => ["png", "jpg", "jpeg", "gif", "ogg", "mo3", "au", "compiled"]
                    .contains(&suffix.as_str()),
                AssetKind::Package => item.path == "main.pak",
            };
            if !classified {
                return Err(invalid(&item.path, "资源分类与扩展名不符或载荷类型被禁止"));
            }
            if !valid_hash(&item.sha256) || !valid_hash(&item.source_sha256) {
                return Err(invalid(
                    &item.path,
                    "来源与内容 sha256 必须是 64 位小写十六进制字符串",
                ));
            }
            if item.kind == AssetKind::Text {
                if !matches!(
                    item.source_encoding.as_deref(),
                    Some("UTF-8" | "UTF-8-BOM" | "UTF-16LE-BOM")
                ) {
                    return Err(invalid(&item.path, "文本来源编码无效"));
                }
            } else if item.size != item.source_size
                || item.sha256 != item.source_sha256
                || item.source_encoding.is_some()
            {
                return Err(invalid(
                    &item.path,
                    "二进制内容必须与来源逐字节一致，且不得声明文本编码",
                ));
            }
            records.push(AssetRecord {
                id,
                kind: item.kind,
                source_size: item.source_size,
                source_sha256: item.source_sha256,
                size: item.size,
                sha256: item.sha256,
                source_encoding: item.source_encoding,
            });
        }
        records.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        let package_records = records
            .iter()
            .filter(|r| r.kind == AssetKind::Package)
            .collect::<Vec<_>>();
        if package_records.len() != 1 {
            return Err(invalid("清单", "必须且只能登记一个参考资源包"));
        }
        let record = package_records[0];
        let package = &raw.package;
        let archive = &raw.source_archive;
        if package.path != record.id.as_str()
            || archive.name != package.path
            || package.entry_count != records.len() as u64 - 1
            || package.index_bytes < 8
            || package.index_bytes >= package.size
            || package.size != record.size
            || package.sha256 != record.sha256
            || archive.size != record.source_size
            || archive.sha256 != record.source_sha256
            || package.extraction_check.trim().is_empty()
        {
            return Err(invalid("清单", "来源、包记录、条目数量或索引范围不一致"));
        }
        Ok(Self {
            baseline_id: raw.baseline_id,
            source_archive: raw.source_archive,
            text_normalization: raw.text_normalization,
            package: raw.package,
            records,
        })
    }
    pub fn baseline_id(&self) -> &str {
        &self.baseline_id
    }
    pub fn source_archive(&self) -> &SourceArchive {
        &self.source_archive
    }
    pub fn text_normalization(&self) -> &str {
        &self.text_normalization
    }
    pub fn package(&self) -> &PackageMetadata {
        &self.package
    }
    pub fn records(&self) -> &[AssetRecord] {
        &self.records
    }
    pub fn record(&self, path: &str) -> Option<&AssetRecord> {
        self.records
            .binary_search_by(|r| r.id.as_str().cmp(path))
            .ok()
            .map(|i| &self.records[i])
    }
    pub fn validate_bytes(&self, path: &str, bytes: &[u8]) -> Result<AuditReport, AssetError> {
        self.record(path)
            .map(|r| r.validate_bytes(bytes))
            .ok_or_else(|| invalid("输入", "资源没有登记在清单中"))
    }
}
/// 宿主目录入口，浏览器应直接使用 `Manifest::parse`。
pub struct AssetRoot;
impl AssetRoot {
    pub fn open(repository_root: &Path) -> Result<AssetIndex, AssetError> {
        check_directory(&repository_root.join("assets"))
            .map_err(|e| io_error("assets/manifest.json", e))?;
        let manifest_path = repository_root.join("assets/manifest.json");
        let metadata = fs::symlink_metadata(&manifest_path)
            .map_err(|e| io_error("assets/manifest.json", e))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(io_error(
                "assets/manifest.json",
                io::Error::from(io::ErrorKind::InvalidInput),
            ));
        }
        let bytes = fs::read(manifest_path).map_err(|e| io_error("assets/manifest.json", e))?;
        Ok(AssetIndex {
            root: repository_root.join("assets/original"),
            manifest: Manifest::parse(&bytes)?,
        })
    }
}
/// 宿主文件系统适配器，所有输入只读。
#[derive(Debug)]
pub struct AssetIndex {
    root: PathBuf,
    manifest: Manifest,
}
impl AssetIndex {
    pub fn len(&self) -> usize {
        self.manifest.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.manifest.records.is_empty()
    }
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    /// 读取登记文件，拒绝根目录或路径组件中的符号链接。
    pub fn read(&self, path: &str) -> Result<Vec<u8>, AssetError> {
        if self.manifest.record(path).is_none() {
            return Err(invalid("输入", "资源没有登记在清单中"));
        }
        self.read_record(path).map_err(|e| io_error(path, e))
    }
    fn read_record(&self, path: &str) -> io::Result<Vec<u8>> {
        check_directory(&self.root)?;
        let mut current = self.root.clone();
        let components = path.split('/').collect::<Vec<_>>();
        for (i, part) in components.iter().enumerate() {
            current.push(part);
            let metadata = fs::symlink_metadata(&current)?;
            if metadata.file_type().is_symlink() {
                return Err(io::Error::from(io::ErrorKind::InvalidInput));
            }
            if i + 1 == components.len() {
                if !metadata.is_file() {
                    return Err(io::Error::from(io::ErrorKind::InvalidInput));
                }
            } else if !metadata.is_dir() {
                return Err(io::Error::from(io::ErrorKind::NotADirectory));
            }
        }
        fs::read(current)
    }
    pub fn audit(&self) -> AuditReport {
        let mut diagnostics = Vec::new();
        let expected = self
            .manifest
            .records
            .iter()
            .map(|r| r.id.as_str().to_owned())
            .collect::<BTreeSet<_>>();
        for record in &self.manifest.records {
            let path = record.id.as_str();
            match self.read_record(path) {
                Ok(bytes) => {
                    diagnostics.extend(record.validate_bytes(&bytes).diagnostics().iter().cloned())
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    diagnostics.push(Diagnostic::Missing {
                        path: path.to_owned(),
                    })
                }
                Err(e) => diagnostics.push(io_diagnostic(path, e.kind())),
            }
        }
        collect_unexpected(&self.root, &self.root, &expected, &mut diagnostics);
        AuditReport::new(diagnostics)
    }
}
fn check_directory(path: &Path) -> io::Result<()> {
    // 根目录的祖先也不能是链接；宿主相对路径由文件系统正常解析。
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        let metadata = fs::symlink_metadata(ancestor)?;
        if metadata.file_type().is_symlink() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
    }
    if !fs::symlink_metadata(path)?.is_dir() {
        return Err(io::Error::from(io::ErrorKind::NotADirectory));
    }
    Ok(())
}
fn collect_unexpected(
    root: &Path,
    directory: &Path,
    expected: &BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let relative = relative_path(root, directory);
    if let Err(e) = check_directory(directory) {
        diagnostics.push(io_diagnostic(&relative, e.kind()));
        return;
    }
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(e) => {
            diagnostics.push(io_diagnostic(&relative, e.kind()));
            return;
        }
    };
    for result in entries {
        let entry = match result {
            Ok(entry) => entry,
            Err(e) => {
                diagnostics.push(io_diagnostic(&relative, e.kind()));
                continue;
            }
        };
        let path = entry.path();
        let relative = relative_path(root, &path);
        match entry.file_type() {
            Ok(kind) if kind.is_symlink() => {
                diagnostics.push(io_diagnostic(&relative, io::ErrorKind::InvalidInput))
            }
            Ok(kind) if kind.is_dir() => collect_unexpected(root, &path, expected, diagnostics),
            Ok(kind) if kind.is_file() => {
                if !expected.contains(&relative) {
                    diagnostics.push(Diagnostic::Unexpected { path: relative });
                }
            }
            Ok(_) => diagnostics.push(io_diagnostic(&relative, io::ErrorKind::InvalidInput)),
            Err(e) => diagnostics.push(io_diagnostic(&relative, e.kind())),
        }
    }
}
fn relative_path(root: &Path, path: &Path) -> String {
    let value = path
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| ".".to_owned());
    if value.is_empty() {
        ".".to_owned()
    } else {
        value
    }
}
fn io_diagnostic(path: &str, kind: io::ErrorKind) -> Diagnostic {
    Diagnostic::Io {
        path: path.to_owned(),
        kind,
        message: "无法读取普通文件或目录（拒绝符号链接）".to_owned(),
    }
}
fn io_error(path: &str, error: io::Error) -> AssetError {
    AssetError::Io {
        path: PathBuf::from(path),
        kind: error.kind(),
        message: "无法读取输入".to_owned(),
    }
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn hex_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn invalid(path: &str, message: &str) -> AssetError {
    AssetError::InvalidRecord {
        path: path.to_owned(),
        message: message.to_owned(),
    }
}
