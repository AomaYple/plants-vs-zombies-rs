use pvz_assets::{AnimationClip, AnimationFrame};

/// 开发仿射合同：kx 决定 y 轴，ky 决定 x 轴，角度单位为度。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    pub xx: f32,
    pub xy: f32,
    pub yx: f32,
    pub yy: f32,
    pub tx: f32,
    pub ty: f32,
}
impl Affine {
    pub fn from_frame(frame: &AnimationFrame) -> Self {
        let kx = frame.kx.unwrap_or(0.0).to_radians();
        let ky = frame.ky.unwrap_or(0.0).to_radians();
        let sx = frame.sx.unwrap_or(1.0);
        let sy = frame.sy.unwrap_or(1.0);
        Self {
            xx: sx * ky.cos(),
            xy: -sy * kx.sin(),
            yx: sx * ky.sin(),
            yy: sy * kx.cos(),
            tx: frame.x.unwrap_or(0.0),
            ty: frame.y.unwrap_or(0.0),
        }
    }
    pub fn transform(self, point: [f32; 2]) -> [f32; 2] {
        [
            self.xx * point[0] + self.xy * point[1] + self.tx,
            self.yx * point[0] + self.yy * point[1] + self.ty,
        ]
    }
}
/// 预装配后的稀疏轨道；图层顺序与源轨道一致。
pub struct PreparedAnimation {
    pub tracks: Vec<Vec<AnimationFrame>>,
    pub names: Vec<String>,
    pub fps: f32,
}
impl PreparedAnimation {
    /// 所选区段的真实图像消费者，保留源轨道索引和顺序。
    pub fn drawable_tracks(&self, segment: &std::ops::Range<usize>) -> Vec<usize> {
        self.tracks
            .iter()
            .enumerate()
            .filter_map(|(index, track)| {
                track
                    .iter()
                    .skip(segment.start)
                    .take(segment.end.saturating_sub(segment.start))
                    .any(|frame| {
                        !frame.f.is_some_and(|n| n < 0.0)
                            && frame.i.as_ref().is_some_and(|image| !image.is_empty())
                    })
                    .then_some(index)
            })
            .collect()
    }
    pub fn new(clip: &AnimationClip) -> Self {
        let tracks = clip.tracks().iter().map(|track| {
            let mut state = AnimationFrame::default();
            track.frames().iter().map(|frame| {
                macro_rules! inherit { ($($field:ident),*) => { $(if frame.$field.is_some() { state.$field.clone_from(&frame.$field); })* }; }
                inherit!(x, y, sx, sy, kx, ky, f, a, i, font, text);
                state.clone()
            }).collect()
        }).collect();
        Self {
            tracks,
            names: clip.tracks().iter().map(|t| t.name().to_owned()).collect(),
            fps: clip.fps(),
        }
    }
    /// 只取 marker 首个非负 f 连续区段；这是明确的开发播放合同。
    pub fn marker_segment(&self, name: &str) -> Option<std::ops::Range<usize>> {
        let index = self.names.iter().position(|n| n == name)?;
        let frames = &self.tracks[index];
        let start = frames.iter().position(|f| f.f.is_some_and(|n| n >= 0.0))?;
        let end = frames[start..]
            .iter()
            .position(|f| f.f.is_some_and(|n| n < 0.0))
            .map_or(frames.len(), |offset| start + offset);
        Some(start..end)
    }
    pub fn frame_at(&self, seconds: f64, segment: &std::ops::Range<usize>) -> usize {
        let length = segment.end.saturating_sub(segment.start);
        if length == 0 || !seconds.is_finite() || seconds < 0.0 {
            return segment.start;
        }
        segment.start + ((seconds * f64::from(self.fps)).floor() % length as f64) as usize
    }
}
