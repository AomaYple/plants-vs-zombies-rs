/// 审计诊断的严重性。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EventKind {
    /// 输入与清单不一致，必须修复。
    Error,
}

/// 资源清单审计发现的问题。
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Diagnostic {
    /// 清单中的文件不存在。
    Missing { path: String },
    /// 文件大小与清单不一致。
    SizeMismatch {
        path: String,
        expected: u64,
        actual: u64,
    },
    /// 文件哈希与清单不一致。
    HashMismatch { path: String },
    /// 目录中存在未登记文件。
    Unexpected { path: String },
    /// 审计读取文件时发生错误。
    Io {
        path: String,
        kind: std::io::ErrorKind,
        message: String,
    },
    /// 文本资源不符合 UTF-8、LF、无 BOM 或末尾换行合同。
    InvalidText { path: String },
}

impl Diagnostic {
    /// 返回诊断严重性。
    pub const fn kind(&self) -> EventKind {
        EventKind::Error
    }

    /// 返回用于确定性排序和脱敏输出的相对 POSIX 路径。
    pub fn path(&self) -> &str {
        match self {
            Self::Missing { path }
            | Self::SizeMismatch { path, .. }
            | Self::HashMismatch { path }
            | Self::Unexpected { path }
            | Self::Io { path, .. }
            | Self::InvalidText { path } => path,
        }
    }
}

/// 一次资源清单审计的稳定结果。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AuditReport {
    diagnostics: Vec<Diagnostic>,
}

impl AuditReport {
    pub(crate) fn new(mut diagnostics: Vec<Diagnostic>) -> Self {
        diagnostics.sort_by(|left, right| left.path().cmp(right.path()));
        Self { diagnostics }
    }

    /// 没有任何错误诊断时返回 `true`。
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// 按稳定路径顺序返回所有诊断。
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
