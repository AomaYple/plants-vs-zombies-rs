use clap::{Parser, Subcommand, ValueEnum, error::ErrorKind};
use pvz_assets::{AssetError, AssetKind, AssetRoot, AuditReport, Diagnostic, parse_reanim};
use serde::Serialize;
use std::{
    io::{self, Write},
    path::Path,
    process::ExitCode,
};

#[derive(Parser)]
#[command(
    name = "pvz-asset-tool",
    version,
    about = "只读核对仓库资源清单，检查登记的 Reanim 动画。",
    subcommand_help_heading = "命令",
    help_template = "{about}\n\n用法：{usage}\n\n{all-args}"
)]
struct Arguments {
    #[command(subcommand)]
    command: Operation,
}

#[derive(Subcommand)]
enum Operation {
    /// 核对资源文件集合、大小、SHA-256 和文本规范。
    Audit {
        /// 资源审计报告格式。
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
    /// 读取登记的 Reanim 资源并报告 CPU 模型。
    InspectReanim {
        /// 资源名称，例如 PeaShooter；不能包含目录或扩展名。
        #[arg(value_name = "NAME", value_parser = parse_name)]
        name: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn parse_name(name: &str) -> Result<String, &'static str> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err("资源名称必须仅包含 ASCII 字母、数字、下划线或连字符");
    }
    Ok(name.to_owned())
}

// 报告不直接序列化库内部枚举，字段顺序与类型代码作为 CLI 的稳定合同。
#[derive(Serialize)]
struct JsonReport<'a> {
    schema_version: u16,
    command: &'static str,
    status: &'static str,
    baseline_id: Option<&'a str>,
    summary: Option<Summary>,
    diagnostics: Vec<JsonDiagnostic<'a>>,
    error: Option<Failure>,
}

#[derive(Serialize)]
struct Summary {
    record_count: usize,
    unpacked_file_count: usize,
    package_count: usize,
    diagnostic_count: usize,
}

#[derive(Serialize)]
struct JsonDiagnostic<'a> {
    code: &'static str,
    path: &'a str,
    message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_size: Option<u64>,
}

impl<'a> From<&'a Diagnostic> for JsonDiagnostic<'a> {
    fn from(diagnostic: &'a Diagnostic) -> Self {
        let (code, message, expected_size, actual_size) = match diagnostic {
            Diagnostic::Missing { .. } => ("missing", "登记资源缺失", None, None),
            Diagnostic::Unexpected { .. } => ("unexpected", "存在未登记的资源文件", None, None),
            Diagnostic::HashMismatch { .. } => {
                ("hash_mismatch", "资源 SHA-256 与清单不一致", None, None)
            }
            Diagnostic::SizeMismatch {
                expected, actual, ..
            } => (
                "size_mismatch",
                "资源大小与清单不一致",
                Some(*expected),
                Some(*actual),
            ),
            Diagnostic::InvalidText { .. } => (
                "invalid_text",
                "文本不符合 UTF-8、LF、无 BOM 或末尾换行规范",
                None,
                None,
            ),
            Diagnostic::Io { .. } => ("io", "无法读取普通文件或目录（拒绝符号链接）", None, None),
        };
        Self {
            code,
            path: diagnostic.path(),
            message,
            expected_size,
            actual_size,
        }
    }
}

#[derive(Serialize)]
struct Failure {
    code: &'static str,
    message: &'static str,
}

impl Failure {
    const fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }

    fn manifest(error: &AssetError) -> Self {
        if matches!(error, AssetError::Io { .. }) {
            Self::new(
                "manifest_read",
                "无法读取 assets/manifest.json，请从仓库根目录运行并检查输入",
            )
        } else {
            Self::new(
                "manifest_invalid",
                "assets/manifest.json 无效或不符合资源清单合同",
            )
        }
    }
}

fn json_output(report: &JsonReport<'_>) -> io::Result<()> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    serde_json::to_writer_pretty(&mut output, report)?;
    writeln!(output)?;
    output.flush()
}

fn audit(format: OutputFormat) -> Result<bool, Failure> {
    let index = match AssetRoot::open(Path::new(".")) {
        Ok(index) => index,
        Err(error) => {
            let failure = Failure::manifest(&error);
            match format {
                OutputFormat::Text => return Err(failure),
                OutputFormat::Json => {
                    json_output(&JsonReport {
                        schema_version: 1,
                        command: "audit",
                        status: "error",
                        baseline_id: None,
                        summary: None,
                        diagnostics: Vec::new(),
                        error: Some(failure),
                    })
                    .map_err(|_| Failure::new("output_failed", "无法写入资源审计报告"))?;
                    return Ok(false);
                }
            }
        }
    };
    let report = index.audit();
    let package_count = index
        .manifest()
        .records()
        .iter()
        .filter(|record| record.kind() == AssetKind::Package)
        .count();
    let summary = Summary {
        record_count: index.len(),
        unpacked_file_count: index.len() - package_count,
        package_count,
        diagnostic_count: report.diagnostics().len(),
    };
    match format {
        OutputFormat::Json => {
            json_output(&JsonReport {
                schema_version: 1,
                command: "audit",
                status: if report.is_clean() { "ok" } else { "error" },
                baseline_id: Some(index.manifest().baseline_id()),
                summary: Some(summary),
                diagnostics: report
                    .diagnostics()
                    .iter()
                    .map(JsonDiagnostic::from)
                    .collect(),
                error: None,
            })
            .map_err(|_| Failure::new("output_failed", "无法写入资源审计报告"))?;
        }
        OutputFormat::Text => text_audit(&summary, &report)
            .map_err(|_| Failure::new("output_failed", "无法写入资源审计报告"))?,
    }
    Ok(report.is_clean())
}

fn text_audit(summary: &Summary, report: &AuditReport) -> io::Result<()> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    writeln!(
        output,
        "资源审计{}：{} 条记录，{} 个解包文件，{} 个来源包，{} 条错误",
        if report.is_clean() {
            "通过"
        } else {
            "失败"
        },
        summary.record_count,
        summary.unpacked_file_count,
        summary.package_count,
        summary.diagnostic_count
    )?;
    for diagnostic in report.diagnostics() {
        let entry = JsonDiagnostic::from(diagnostic);
        write!(output, "[{}] {}：{}", entry.code, entry.path, entry.message)?;
        if let (Some(expected), Some(actual)) = (entry.expected_size, entry.actual_size) {
            write!(output, "（预期 {expected} 字节，实际 {actual} 字节）")?;
        }
        writeln!(output)?;
    }
    output.flush()
}

fn inspect(name: &str) -> Result<bool, Failure> {
    let index = AssetRoot::open(Path::new(".")).map_err(|error| Failure::manifest(&error))?;
    let path = format!("reanim/{name}.reanim");
    let record = index
        .manifest()
        .record(&path)
        .ok_or_else(|| Failure::new("asset_unregistered", "指定的 Reanim 资源未登记在清单中"))?;
    let bytes = index
        .read(&path)
        .map_err(|_| Failure::new("asset_read", "无法读取登记的 Reanim 资源"))?;
    if !record.validate_bytes(&bytes).is_clean() {
        return Err(Failure::new(
            "asset_invalid",
            "Reanim 资源指纹或文本规范与清单不一致",
        ));
    }
    let source = std::str::from_utf8(&bytes)
        .map_err(|_| Failure::new("asset_invalid", "Reanim 资源不是 UTF-8 文本"))?;
    // 格式错误可能来自不可信标签，不回显解析器消息或源文本。
    let clip = parse_reanim(source)
        .map_err(|_| Failure::new("reanim_invalid", "Reanim 文本无法解析为有效的 CPU 动画模型"))?;
    let mut output = io::BufWriter::new(io::stdout().lock());
    let result = (|| {
        writeln!(output, "Reanim 资源：{path}")?;
        writeln!(
            output,
            "帧率：{}；轨道：{}",
            clip.fps(),
            clip.tracks().len()
        )?;
        for track in clip.tracks() {
            writeln!(output, "{}：{} 帧", track.name(), track.frames().len())?;
        }
        output.flush()
    })();
    result.map_err(|_: io::Error| Failure::new("output_failed", "无法写入 Reanim 模型报告"))?;
    Ok(true)
}

fn main() -> ExitCode {
    let arguments = match Arguments::try_parse() {
        Ok(arguments) => arguments,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            return if error.print().is_ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            };
        }
        Err(_) => {
            // clap 默认参数错误会回显用户输入；统一诊断避免泄露传入的绝对路径。
            // 诊断管道可能已关闭；输出失败不能把既定退出码改成 panic。
            let _ = writeln!(
                io::stderr().lock(),
                "[arguments_invalid] 命令参数无效，请运行 pvz-asset-tool --help 查看用法"
            );
            return ExitCode::from(2);
        }
    };
    match match arguments.command {
        Operation::Audit { format } => audit(format),
        Operation::InspectReanim { name } => inspect(&name),
    } {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(failure) => {
            let _ = writeln!(
                io::stderr().lock(),
                "[{}] {}",
                failure.code,
                failure.message
            );
            ExitCode::from(1)
        }
    }
}
