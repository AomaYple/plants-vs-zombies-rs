use crate::{AssetError, xml};
use std::collections::BTreeSet;

/// 粒子描述树，保留多个发射器及曲线原文，不宣称实现粒子运行语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleSystem {
    emitters: Vec<ParticleEmitter>,
}
impl ParticleSystem {
    pub fn emitters(&self) -> &[ParticleEmitter] {
        &self.emitters
    }
}
/// 一个完整发射器；实际资源允许 Name、Image 和 EmitterType 缺省。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleEmitter {
    properties: Vec<ParticleProperty>,
}
impl ParticleEmitter {
    pub fn properties(&self) -> &[ParticleProperty] {
        &self.properties
    }
    /// 返回首个同名属性，重复的 Field 与 SystemField 请遍历 `properties`。
    pub fn property(&self, name: &str) -> Option<&ParticleProperty> {
        self.properties.iter().find(|p| p.name == name)
    }
    pub fn name(&self) -> Option<&str> {
        self.property("Name").map(ParticleProperty::text)
    }
    pub fn image(&self) -> Option<&str> {
        self.property("Image").map(ParticleProperty::text)
    }
    pub fn emitter_type(&self) -> Option<&str> {
        self.property("EmitterType").map(ParticleProperty::text)
    }
}
/// 标量或 Field/SystemField 子树，曲线字符串只做结构校验。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleProperty {
    name: String,
    text: String,
    children: Vec<ParticleProperty>,
}
impl ParticleProperty {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn children(&self) -> &[ParticleProperty] {
        &self.children
    }
}
const SCALARS: &[&str] = &[
    "Name",
    "Image",
    "EmitterType",
    "ImageCol",
    "ImageRow",
    "ImageFrames",
    "Animated",
    "AnimationRate",
    "SystemDuration",
    "SystemLoops",
    "SystemAlpha",
    "ParticleDuration",
    "ParticleLoops",
    "ParticleAlpha",
    "ParticleScale",
    "ParticleStretch",
    "ParticleRed",
    "ParticleGreen",
    "ParticleBlue",
    "ParticleBrightness",
    "ParticleSpinAngle",
    "ParticleSpinSpeed",
    "SpawnRate",
    "SpawnMinActive",
    "SpawnMaxActive",
    "SpawnMaxLaunched",
    "LaunchSpeed",
    "LaunchAngle",
    "EmitterRadius",
    "EmitterBoxX",
    "EmitterBoxY",
    "EmitterOffsetX",
    "EmitterOffsetY",
    "EmitterSkewX",
    "RandomLaunchSpin",
    "AlignLaunchSpin",
    "Additive",
    "CollisionReflect",
    "CollisionSpin",
    "ParticlesDontFollow",
    "DieIfOverloaded",
    "FullScreen",
    "CrossFadeDuration",
    "RandomStartTime",
    "HardwareOnly",
    "ClipTop",
];
fn property(node: xml::Node) -> Result<ParticleProperty, AssetError> {
    if node.tag == "Field" || node.tag == "SystemField" {
        xml::container(&node)?;
        let mut seen = BTreeSet::new();
        let mut children = Vec::new();
        for child in node.children {
            if !matches!(child.tag.as_str(), "FieldType" | "X" | "Y") {
                return Err(xml::error("Field 中存在未知或错误上下文的标签"));
            }
            if !seen.insert(child.tag.clone()) {
                return Err(xml::error("Field 的标量字段重复"));
            }
            xml::scalar(&child)?;
            if child.text.is_empty() {
                return Err(xml::error("Field 的标量字段不得为空"));
            }
            children.push(ParticleProperty {
                name: child.tag,
                text: child.text,
                children: Vec::new(),
            });
        }
        if !seen.contains("FieldType") {
            return Err(xml::error("Field 缺少 FieldType"));
        }
        Ok(ParticleProperty {
            name: node.tag,
            text: node.text,
            children,
        })
    } else {
        if !SCALARS.contains(&node.tag.as_str()) {
            return Err(xml::error("Emitter 中存在未知或错误上下文的标签"));
        }
        xml::scalar(&node)?;
        if node.text.is_empty() {
            return Err(xml::error("Emitter 的标量字段不得为空"));
        }
        Ok(ParticleProperty {
            name: node.tag,
            text: node.text,
            children: Vec::new(),
        })
    }
}
pub(crate) fn parse(source: &str) -> Result<ParticleSystem, AssetError> {
    let roots = xml::parse(source)?;
    let mut emitters = Vec::new();
    for node in roots {
        if node.tag != "Emitter" {
            return Err(xml::error("粒子顶层必须是 Emitter"));
        }
        xml::container(&node)?;
        if node.children.is_empty() {
            return Err(xml::error("Emitter 不能为空"));
        }
        let mut seen = BTreeSet::new();
        let mut properties = Vec::new();
        for child in node.children {
            if !matches!(child.tag.as_str(), "Field" | "SystemField")
                && !seen.insert(child.tag.clone())
            {
                return Err(xml::error("Emitter 的标量字段重复"));
            }
            let value = property(child)?;
            properties.push(value);
        }
        emitters.push(ParticleEmitter { properties });
    }
    Ok(ParticleSystem { emitters })
}
