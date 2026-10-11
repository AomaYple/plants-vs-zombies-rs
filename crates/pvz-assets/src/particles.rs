use crate::manifest::AssetError;
use quick_xml::{Reader, events::Event};

/// 粒子系统 XML 的最小模型。
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleSystem {
    name: String,
    image: String,
    emitter_type: String,
    system_duration: Option<f32>,
    fields: Vec<String>,
}

impl ParticleSystem {
    /// 创建粒子系统夹具或解析结果。
    pub fn new(
        name: impl Into<String>,
        image: impl Into<String>,
        emitter_type: impl Into<String>,
        system_duration: Option<f32>,
        fields: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            image: image.into(),
            emitter_type: emitter_type.into(),
            system_duration,
            fields,
        }
    }

    /// 返回系统名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 返回图像资源 ID。
    pub fn image(&self) -> &str {
        &self.image
    }

    /// 返回发射器类型。
    pub fn emitter_type(&self) -> &str {
        &self.emitter_type
    }

    /// 返回系统持续时间。
    pub const fn system_duration(&self) -> Option<f32> {
        self.system_duration
    }

    /// 返回 FieldType 列表。
    pub fn fields(&self) -> &[String] {
        &self.fields
    }
}

pub(crate) fn parse(source: &str) -> Result<ParticleSystem, AssetError> {
    let mut reader = Reader::from_str(source);
    reader.config_mut().trim_text(true);
    let mut current_tag = None;
    let mut field_depth = 0_u32;
    let mut name = None;
    let mut image = None;
    let mut emitter_type = None;
    let mut system_duration = None;
    let mut fields = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(start)) => {
                let tag = String::from_utf8_lossy(start.name().as_ref()).into_owned();
                if tag == "Field" {
                    field_depth += 1;
                }
                if matches!(
                    tag.as_str(),
                    "Name" | "Image" | "EmitterType" | "SystemDuration" | "FieldType"
                ) {
                    current_tag = Some(tag);
                }
            }
            Ok(Event::Text(text)) => {
                let value = text
                    .decode()
                    .map_err(|error| AssetError::Xml {
                        message: error.to_string(),
                    })?
                    .into_owned();
                match current_tag.as_deref() {
                    Some("Name") => name = Some(value),
                    Some("Image") => image = Some(value),
                    Some("EmitterType") => emitter_type = Some(value),
                    Some("SystemDuration") => {
                        let duration = value.parse::<f32>().map_err(|error| AssetError::Xml {
                            message: format!("SystemDuration 无效：{error}"),
                        })?;
                        if !duration.is_finite() || duration < 0.0 {
                            return Err(AssetError::Xml {
                                message: "SystemDuration 必须是非负有限数".to_owned(),
                            });
                        }
                        system_duration = Some(duration);
                    }
                    Some("FieldType") if field_depth > 0 => fields.push(value),
                    _ => {}
                }
            }
            Ok(Event::End(end)) => {
                let tag = String::from_utf8_lossy(end.name().as_ref()).into_owned();
                if tag == "Field" {
                    field_depth = field_depth.saturating_sub(1);
                }
                if current_tag.as_deref() == Some(tag.as_str()) {
                    current_tag = None;
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => {
                return Err(AssetError::Xml {
                    message: error.to_string(),
                });
            }
            _ => {}
        }
    }

    let name = name
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AssetError::Xml {
            message: "缺少 Name".to_owned(),
        })?;
    let image = image
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AssetError::Xml {
            message: "缺少 Image".to_owned(),
        })?;
    let emitter_type = emitter_type
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AssetError::Xml {
            message: "缺少 EmitterType".to_owned(),
        })?;
    Ok(ParticleSystem::new(
        name,
        image,
        emitter_type,
        system_duration,
        fields,
    ))
}
