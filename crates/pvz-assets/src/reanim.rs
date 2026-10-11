use crate::manifest::AssetError;
use quick_xml::{Reader, events::Event};

/// Reanim 动画片段的最小模型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationClip {
    fps: u32,
    tracks: Vec<AnimationTrack>,
}

impl AnimationClip {
    /// 返回资源时间轴声明的帧率。
    pub const fn fps(&self) -> u32 {
        self.fps
    }

    /// 返回按文本顺序排列的轨道。
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }
}

/// Reanim 单条轨道的最小模型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationTrack {
    name: String,
    frame_count: usize,
}

impl AnimationTrack {
    /// 返回轨道名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 返回轨道中的时间点数量。
    pub const fn frame_count(&self) -> usize {
        self.frame_count
    }
}

struct TrackBuilder {
    name: Option<String>,
    frame_count: usize,
}

pub(crate) fn parse(source: &str) -> Result<AnimationClip, AssetError> {
    let mut reader = Reader::from_str(source);
    reader.config_mut().trim_text(true);
    let mut fps = None;
    let mut tracks = Vec::new();
    let mut current_track: Option<TrackBuilder> = None;
    let mut text_tag = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(start)) => {
                let tag = String::from_utf8_lossy(start.name().as_ref()).into_owned();
                match tag.as_str() {
                    "track" => {
                        current_track = Some(TrackBuilder {
                            name: None,
                            frame_count: 0,
                        })
                    }
                    "t" => {
                        if let Some(track) = current_track.as_mut() {
                            track.frame_count += 1;
                        }
                    }
                    "fps" | "name" => text_tag = Some(tag),
                    _ => {}
                }
            }
            Ok(Event::Empty(empty)) => {
                if empty.name().as_ref() == b"t"
                    && let Some(track) = current_track.as_mut()
                {
                    track.frame_count += 1;
                }
            }
            Ok(Event::Text(text)) => {
                let value = text
                    .decode()
                    .map_err(|error| AssetError::Xml {
                        message: error.to_string(),
                    })?
                    .into_owned();
                match text_tag.as_deref() {
                    Some("fps") => {
                        fps = Some(value.parse::<u32>().map_err(|error| AssetError::Xml {
                            message: format!("fps 无效：{error}"),
                        })?);
                    }
                    Some("name") => {
                        if let Some(track) = current_track.as_mut() {
                            track.name = Some(value);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(end)) => {
                let tag = String::from_utf8_lossy(end.name().as_ref()).into_owned();
                if tag == "track" {
                    let track = current_track.take().ok_or_else(|| AssetError::Xml {
                        message: "轨道结束顺序无效".to_owned(),
                    })?;
                    let name = track.name.ok_or_else(|| AssetError::Xml {
                        message: "轨道缺少 name".to_owned(),
                    })?;
                    tracks.push(AnimationTrack {
                        name,
                        frame_count: track.frame_count,
                    });
                }
                if text_tag.as_deref() == Some(tag.as_str()) {
                    text_tag = None;
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

    let fps = fps
        .filter(|value| *value > 0)
        .ok_or_else(|| AssetError::Xml {
            message: "缺少有效 fps".to_owned(),
        })?;
    if tracks.is_empty() {
        return Err(AssetError::Xml {
            message: "没有轨道".to_owned(),
        });
    }
    Ok(AnimationClip { fps, tracks })
}
