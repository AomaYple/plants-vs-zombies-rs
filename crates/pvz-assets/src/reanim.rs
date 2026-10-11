use crate::{AssetError, xml};
use std::collections::BTreeSet;

/// Reanim 动画片段；原始稀疏帧由应用层按资源时基装配。
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationClip {
    fps: f32,
    tracks: Vec<AnimationTrack>,
}
impl AnimationClip {
    pub const fn fps(&self) -> f32 {
        self.fps
    }
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }
}
/// 一条按文本顺序保存的动画轨道。
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationTrack {
    name: String,
    frames: Vec<AnimationFrame>,
}
impl AnimationTrack {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
    pub fn frames(&self) -> &[AnimationFrame] {
        &self.frames
    }
}
/// 原始稀疏帧；缺省与显式空字符串分开保留，不在解析时推断继承。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AnimationFrame {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub sx: Option<f32>,
    pub sy: Option<f32>,
    pub kx: Option<f32>,
    pub ky: Option<f32>,
    pub f: Option<f32>,
    pub a: Option<f32>,
    pub i: Option<String>,
    pub font: Option<String>,
    pub text: Option<String>,
}
fn frame(node: &xml::Node) -> Result<AnimationFrame, AssetError> {
    xml::container(node)?;
    let mut result = AnimationFrame::default();
    let mut seen = BTreeSet::new();
    for child in &node.children {
        if !seen.insert(child.tag.as_str()) {
            return Err(xml::error(format!("帧字段 {} 重复", child.tag)));
        }
        let value = xml::scalar(child)?;
        match child.tag.as_str() {
            "x" => result.x = Some(xml::finite(value, "x")?),
            "y" => result.y = Some(xml::finite(value, "y")?),
            "sx" => result.sx = Some(xml::finite(value, "sx")?),
            "sy" => result.sy = Some(xml::finite(value, "sy")?),
            "kx" => result.kx = Some(xml::finite(value, "kx")?),
            "ky" => result.ky = Some(xml::finite(value, "ky")?),
            "f" => result.f = Some(xml::finite(value, "f")?),
            "a" => result.a = Some(xml::finite(value, "a")?),
            "i" => result.i = Some(value.to_owned()),
            "font" => result.font = Some(value.to_owned()),
            "text" => result.text = Some(value.to_owned()),
            _ => return Err(xml::error("帧中存在未知或错误上下文的字段")),
        }
    }
    Ok(result)
}
fn track(node: &xml::Node) -> Result<AnimationTrack, AssetError> {
    xml::container(node)?;
    let mut name = None;
    let mut frames = Vec::new();
    for child in &node.children {
        match child.tag.as_str() {
            "name" => {
                if name.is_some() {
                    return Err(xml::error("轨道 name 重复"));
                }
                let value = xml::scalar(child)?;
                if value.is_empty() {
                    return Err(xml::error("轨道 name 不得为空"));
                }
                name = Some(value.to_owned());
            }
            "t" => frames.push(frame(child)?),
            _ => return Err(xml::error("轨道中存在未知或错误上下文的字段")),
        }
    }
    let name = name.ok_or_else(|| xml::error("轨道缺少 name"))?;
    if frames.is_empty() {
        return Err(xml::error("轨道没有时间点"));
    }
    Ok(AnimationTrack { name, frames })
}
pub(crate) fn parse(source: &str) -> Result<AnimationClip, AssetError> {
    let mut roots = xml::parse(source)?;
    // 兼容原创夹具的显式根标签；原版为 fps 与 track 的顶层片段。
    if roots.len() == 1 && roots[0].tag == "root" {
        let root = roots.remove(0);
        xml::container(&root)?;
        roots = root.children;
    }
    let mut fps = None;
    let mut tracks = Vec::new();
    for node in roots {
        match node.tag.as_str() {
            "fps" => {
                if fps.is_some() {
                    return Err(xml::error("fps 重复"));
                }
                let value = xml::finite(xml::scalar(&node)?, "fps")?;
                if value <= 0.0 {
                    return Err(xml::error("fps 必须是正有限数"));
                }
                fps = Some(value);
            }
            "track" => tracks.push(track(&node)?),
            _ => return Err(xml::error("Reanim 顶层存在未知或错误上下文的标签")),
        }
    }
    let fps = fps.ok_or_else(|| xml::error("缺少 fps"))?;
    if tracks.is_empty() {
        return Err(xml::error("没有轨道"));
    }
    Ok(AnimationClip { fps, tracks })
}
