use crate::{
    animation::PreparedAnimation,
    audio::{AudioInfo, validate_audio},
};
use pvz_assets::{Manifest, parse_reanim};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub const BACKGROUND: &str = "images/background1.jpg";
pub const ANIMATION: &str = "reanim/PeaShooter.reanim";
const ZOMBIE_ANIMATION: &str = "reanim/Zombie.reanim";
const SUNFLOWER_ANIMATION: &str = "reanim/SunFlower.reanim";
pub const EFFECT: &str = "sounds/seedlift.ogg";
pub const PEA_IMAGE: &str = "images/ProjectilePea.png";
pub const MUSIC: &str = "sounds/ZombiesOnYourLawn.ogg";
/// 开发默认播放片段；必须用登记 XML 确认包含整个植物。
pub const DEVELOPMENT_IDLE_MARKER: &str = "anim_full_idle";
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum VisualKind {
    Sunflower,
    Peashooter,
    Zombie,
    Pea,
    Sun,
    LawnMower,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum SoundCue {
    Plant,
    Collect,
    Hit,
    Mower,
}
pub const SOUND_SOURCES: &[(SoundCue, &str)] = &[
    (SoundCue::Plant, "sounds/plant.ogg"),
    (SoundCue::Collect, "sounds/points.ogg"),
    (SoundCue::Hit, "sounds/splat.ogg"),
    (SoundCue::Mower, "sounds/lawnmower.ogg"),
];
/// 从登记 XML 选择普通僵尸 walk 身体；排除附件是开发构图合同。
pub const ZOMBIE_TRACKS: &[&str] = &[
    "anim_innerarm3",
    "anim_innerarm2",
    "anim_innerarm1",
    "Zombie_neck",
    "anim_head1",
    "Zombie_innerleg_upper",
    "Zombie_innerleg_lower",
    "Zombie_innerleg_foot",
    "Zombie_outerleg_upper",
    "Zombie_outerleg_foot",
    "Zombie_outerleg_lower",
    "Zombie_body",
    "Zombie_tie",
    "anim_head2",
    "anim_tongue",
    "Zombie_outerarm_hand",
    "Zombie_outerarm_upper",
    "Zombie_outerarm_lower",
    "anim_hair",
];
pub const POSE_SOURCES: &[(VisualKind, &str, usize, &[&str])] = &[
    (VisualKind::LawnMower, "reanim/LawnMower.reanim", 0, &[]),
    (VisualKind::Sun, "reanim/Sun.reanim", 0, &[]),
];
/// 静态构图继续使用稀疏继承及源图层顺序，不猜测附件播放器语义。
pub fn selected_pose(
    animation: &PreparedAnimation,
    frame: usize,
    selected: &[&str],
) -> Result<Vec<pvz_assets::AnimationFrame>, ResourceError> {
    if selected
        .iter()
        .any(|name| !animation.names.iter().any(|n| n == name))
    {
        return Err(error("构图", "指定轨道不存在"));
    }
    let pose: Vec<_> = animation
        .names
        .iter()
        .zip(&animation.tracks)
        .filter(|(name, _)| selected.is_empty() || selected.contains(&name.as_str()))
        .filter_map(|(_, track)| track.get(frame))
        .filter(|frame| {
            !frame.f.is_some_and(|n| n < 0.0) && frame.i.as_ref().is_some_and(|id| !id.is_empty())
        })
        .cloned()
        .collect();
    if pose.is_empty() {
        return Err(error("构图", "指定帧没有可绘制图像"));
    }
    Ok(pose)
}
/// 已逐项核对的开发映射；resources.xml 没有登记 PeaShooter 图像。
pub const IMAGE_MAPPING: &[(&str, &str)] = &[
    ("IMAGE_REANIM_LAWNMOWER_BODY", "reanim/LawnMower_body.png"),
    (
        "IMAGE_REANIM_LAWNMOWER_BODY_TRICKED",
        "reanim/LawnMower_body_tricked.png",
    ),
    (
        "IMAGE_REANIM_LAWNMOWER_DICE_TRICKED",
        "reanim/LawnMower_dice_tricked.png",
    ),
    (
        "IMAGE_REANIM_LAWNMOWER_ENGINE",
        "reanim/LawnMower_engine.png",
    ),
    (
        "IMAGE_REANIM_LAWNMOWER_ENGINE_TRICKED",
        "reanim/LawnMower_engine_tricked.png",
    ),
    (
        "IMAGE_REANIM_LAWNMOWER_EXHAUST",
        "reanim/LawnMower_exhaust.png",
    ),
    (
        "IMAGE_REANIM_LAWNMOWER_EXHAUST_TRICKED",
        "reanim/LawnMower_exhaust_tricked.png",
    ),
    ("IMAGE_REANIM_LAWNMOWER_PULL", "reanim/LawnMower_pull.png"),
    (
        "IMAGE_REANIM_LAWNMOWER_WHEEL1",
        "reanim/LawnMower_wheel1.png",
    ),
    (
        "IMAGE_REANIM_LAWNMOWER_WHEEL2",
        "reanim/LawnMower_wheel2.png",
    ),
    (
        "IMAGE_REANIM_LAWNMOWER_WHEELPIECE",
        "reanim/LawnMower_wheelpiece.png",
    ),
    (
        "IMAGE_REANIM_LAWNMOWER_WHEELSHINE",
        "reanim/LawnMower_wheelshine.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_BACKLEAF",
        "reanim/PeaShooter_backleaf.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_BACKLEAF_LEFTTIP",
        "reanim/PeaShooter_backleaf_lefttip.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_BACKLEAF_RIGHTTIP",
        "reanim/PeaShooter_backleaf_righttip.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_BLINK1",
        "reanim/PeaShooter_blink1.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_BLINK2",
        "reanim/PeaShooter_blink2.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_EYEBROW",
        "reanim/PeaShooter_eyebrow.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_FRONTLEAF",
        "reanim/PeaShooter_frontleaf.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_FRONTLEAF_LEFTTIP",
        "reanim/PeaShooter_frontleaf_lefttip.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_FRONTLEAF_RIGHTTIP",
        "reanim/PeaShooter_frontleaf_righttip.png",
    ),
    ("IMAGE_REANIM_PEASHOOTER_HEAD", "reanim/PeaShooter_Head.png"),
    (
        "IMAGE_REANIM_PEASHOOTER_HEADLEAF_2RDFARTHEST",
        "reanim/PeaShooter_headleaf_2rdfarthest.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_HEADLEAF_3RDFARTHEST",
        "reanim/PeaShooter_headleaf_3rdfarthest.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_HEADLEAF_FARTHEST",
        "reanim/PeaShooter_headleaf_farthest.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_HEADLEAF_NEAREST",
        "reanim/PeaShooter_headleaf_nearest.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_HEADLEAF_TIP_BOTTOM",
        "reanim/PeaShooter_headleaf_tip_bottom.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_HEADLEAF_TIP_TOP",
        "reanim/PeaShooter_headleaf_tip_top.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_MOUTH",
        "reanim/PeaShooter_mouth.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_STALK_BOTTOM",
        "reanim/PeaShooter_stalk_bottom.png",
    ),
    (
        "IMAGE_REANIM_PEASHOOTER_STALK_TOP",
        "reanim/PeaShooter_stalk_top.png",
    ),
    ("IMAGE_REANIM_SUN1", "reanim/Sun1.png"),
    ("IMAGE_REANIM_SUN2", "reanim/Sun2.png"),
    ("IMAGE_REANIM_SUN3", "reanim/Sun3.png"),
    (
        "IMAGE_REANIM_SUNFLOWER_BLINK1",
        "reanim/SunFlower_blink1.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_BLINK2",
        "reanim/SunFlower_blink2.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_BOTTOMPETALS",
        "reanim/SunFlower_bottompetals.png",
    ),
    ("IMAGE_REANIM_SUNFLOWER_HEAD", "reanim/SunFlower_head.png"),
    (
        "IMAGE_REANIM_SUNFLOWER_LEFTPETAL1",
        "reanim/SunFlower_leftpetal1.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_LEFTPETAL2",
        "reanim/SunFlower_leftpetal2.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_LEFTPETAL3",
        "reanim/SunFlower_leftpetal3.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_LEFTPETAL4",
        "reanim/SunFlower_leftpetal4.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_LEFTPETAL5",
        "reanim/SunFlower_leftpetal5.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_LEFTPETAL6",
        "reanim/SunFlower_leftpetal6.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_LEFTPETAL7",
        "reanim/SunFlower_leftpetal7.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_LEFTPETAL8",
        "reanim/SunFlower_leftpetal8.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_RIGHTPETAL1",
        "reanim/SunFlower_rightpetal1.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_RIGHTPETAL2",
        "reanim/SunFlower_rightpetal2.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_RIGHTPETAL3",
        "reanim/SunFlower_rightpetal3.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_RIGHTPETAL4",
        "reanim/SunFlower_rightpetal4.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_RIGHTPETAL5",
        "reanim/SunFlower_rightpetal5.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_RIGHTPETAL6",
        "reanim/SunFlower_rightpetal6.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_RIGHTPETAL7",
        "reanim/SunFlower_rightpetal7.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_RIGHTPETAL8",
        "reanim/SunFlower_rightpetal8.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_RIGHTPETAL9",
        "reanim/SunFlower_rightpetal9.png",
    ),
    (
        "IMAGE_REANIM_SUNFLOWER_TOPPETALS",
        "reanim/SunFlower_toppetals.png",
    ),
    ("IMAGE_REANIM_ZOMBIE_BODY", "reanim/Zombie_body.png"),
    ("IMAGE_REANIM_ZOMBIE_BUCKET1", "reanim/Zombie_bucket1.png"),
    ("IMAGE_REANIM_ZOMBIE_CONE1", "reanim/Zombie_cone1.png"),
    (
        "IMAGE_REANIM_ZOMBIE_DUCKYTUBE",
        "reanim/Zombie_duckytube.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_DUCKYTUBE_INWATER",
        "reanim/Zombie_duckytube_inwater.png",
    ),
    ("IMAGE_REANIM_ZOMBIE_FLAGHAND", "reanim/Zombie_flaghand.png"),
    ("IMAGE_REANIM_ZOMBIE_HAIR", "reanim/Zombie_hair.png"),
    ("IMAGE_REANIM_ZOMBIE_HEAD", "reanim/Zombie_head.png"),
    (
        "IMAGE_REANIM_ZOMBIE_INNERARM_HAND",
        "reanim/Zombie_innerarm_hand.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_INNERARM_LOWER",
        "reanim/Zombie_innerarm_lower.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_INNERARM_SCREENDOOR",
        "reanim/Zombie_innerarm_screendoor.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_INNERARM_SCREENDOOR_HAND",
        "reanim/Zombie_innerarm_screendoor_hand.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_INNERARM_UPPER",
        "reanim/Zombie_innerarm_upper.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_INNERLEG_FOOT",
        "reanim/Zombie_innerleg_foot.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_INNERLEG_LOWER",
        "reanim/Zombie_innerleg_lower.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_INNERLEG_UPPER",
        "reanim/Zombie_innerleg_upper.png",
    ),
    ("IMAGE_REANIM_ZOMBIE_JAW", "reanim/Zombie_jaw.png"),
    (
        "IMAGE_REANIM_ZOMBIE_MUSTACHE1",
        "reanim/Zombie_mustache1.png",
    ),
    ("IMAGE_REANIM_ZOMBIE_NECK", "reanim/Zombie_neck.png"),
    (
        "IMAGE_REANIM_ZOMBIE_OUTERARM_HAND",
        "reanim/Zombie_outerarm_hand.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_OUTERARM_HAND2",
        "reanim/Zombie_outerarm_hand2.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_OUTERARM_LOWER",
        "reanim/Zombie_outerarm_lower.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_OUTERARM_SCREENDOOR",
        "reanim/Zombie_outerarm_screendoor.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_OUTERARM_UPPER",
        "reanim/Zombie_outerarm_upper.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_OUTERARM_UPPER2",
        "reanim/Zombie_outerarm_upper2.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_OUTERLEG_FOOT",
        "reanim/Zombie_outerleg_foot.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_OUTERLEG_LOWER",
        "reanim/Zombie_outerleg_lower.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_OUTERLEG_UPPER",
        "reanim/Zombie_outerleg_upper.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_SCREENDOOR1",
        "reanim/Zombie_screendoor1.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_SNORKLE_WHITEWATER1",
        "reanim/Zombie_snorkle_whitewater1.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_SNORKLE_WHITEWATER2",
        "reanim/Zombie_snorkle_whitewater2.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_SNORKLE_WHITEWATER3",
        "reanim/Zombie_snorkle_whitewater3.png",
    ),
    ("IMAGE_REANIM_ZOMBIE_TIE", "reanim/Zombie_tie.png"),
    ("IMAGE_REANIM_ZOMBIE_TONGUE", "reanim/Zombie_tongue.png"),
    (
        "IMAGE_REANIM_ZOMBIE_WHITEWATER1",
        "reanim/Zombie_whitewater1.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_WHITEWATER2",
        "reanim/Zombie_whitewater2.png",
    ),
    (
        "IMAGE_REANIM_ZOMBIE_WHITEWATER3",
        "reanim/Zombie_whitewater3.png",
    ),
];

/// 仅输出登记的相对路径，不透传平台错误或用户输入。
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResourceError {
    pub path: String,
    pub reason: &'static str,
}
impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "资源加载失败：{}：{}", self.path, self.reason)
    }
}
impl std::error::Error for ResourceError {}
fn error(path: &str, reason: &'static str) -> ResourceError {
    let safe = pvz_data::AssetId::new(path).is_ok();
    ResourceError {
        path: if safe {
            path.to_owned()
        } else {
            "〈无效资源标识〉".to_owned()
        },
        reason,
    }
}
pub fn validated_bytes(
    manifest: &Manifest,
    path: &str,
    bytes: Vec<u8>,
) -> Result<Vec<u8>, ResourceError> {
    let report = manifest
        .validate_bytes(path, &bytes)
        .map_err(|_| error("清单", "输入未登记或路径无效"))?;
    if !report.is_clean() {
        return Err(error(path, "大小、SHA-256 或文本规范与清单不符"));
    }
    Ok(bytes)
}

/// native 只读清单入口和 Web 下载入口共享完全相同的字节校验合同。
pub struct AssetProvider {
    #[cfg(not(target_arch = "wasm32"))]
    index: pvz_assets::AssetIndex,
    #[cfg(target_arch = "wasm32")]
    manifest: Manifest,
}
impl AssetProvider {
    pub async fn open() -> Result<Self, ResourceError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let index = pvz_assets::AssetRoot::open(std::path::Path::new("."))
                .map_err(|_| error("assets/manifest.json", "无法读取或解析清单"))?;
            Ok(Self { index })
        }
        #[cfg(target_arch = "wasm32")]
        {
            let bytes = fetch("/assets/manifest.json")
                .await
                .map_err(|_| error("assets/manifest.json", "浏览器下载清单失败"))?;
            let manifest = Manifest::parse(&bytes)
                .map_err(|_| error("assets/manifest.json", "清单结构无效"))?;
            Ok(Self { manifest })
        }
    }
    pub fn manifest(&self) -> &Manifest {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.index.manifest()
        }
        #[cfg(target_arch = "wasm32")]
        {
            &self.manifest
        }
    }
    pub async fn read(&self, path: &str) -> Result<Vec<u8>, ResourceError> {
        if self.manifest().record(path).is_none() {
            return Err(error("清单", "输入未登记或路径无效"));
        }
        #[cfg(not(target_arch = "wasm32"))]
        let bytes = self.native_bytes(path)?;
        #[cfg(target_arch = "wasm32")]
        let bytes = fetch(&format!("/assets/original/{path}"))
            .await
            .map_err(|_| error(path, "浏览器下载资源失败"))?;
        validated_bytes(self.manifest(), path, bytes)
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn native_bytes(&self, path: &str) -> Result<Vec<u8>, ResourceError> {
        if self.manifest().record(path).is_none() {
            return Err(error("清单", "输入未登记或路径无效"));
        }
        self.index
            .read(path)
            .map_err(|_| error(path, "文件缺失或无法只读访问"))
    }
}
#[cfg(target_arch = "wasm32")]
async fn fetch(path: &str) -> Result<Vec<u8>, ()> {
    let response = gloo_net::http::Request::get(path)
        .send()
        .await
        .map_err(|_| ())?;
    if !response.ok() {
        return Err(());
    }
    response.binary().await.map_err(|_| ())
}
/// 只接受登记映射，每个动画图像引用必须精确命中一次。
pub fn resolve_images(
    clip: &pvz_assets::AnimationClip,
    manifest: &Manifest,
) -> Result<BTreeMap<String, String>, ResourceError> {
    let ids = clip
        .tracks()
        .iter()
        .flat_map(|t| t.frames())
        .filter_map(|f| f.i.as_deref())
        .filter(|id| !id.is_empty())
        .collect::<BTreeSet<_>>();
    ids.into_iter()
        .map(|id| {
            let paths = IMAGE_MAPPING
                .iter()
                .filter(|(candidate, _)| *candidate == id)
                .map(|(_, path)| *path)
                .collect::<Vec<_>>();
            if paths.len() != 1 || manifest.record(paths[0]).is_none() {
                return Err(error(ANIMATION, "图像引用没有唯一登记映射"));
            }
            Ok((id.to_owned(), paths[0].to_owned()))
        })
        .collect()
}
pub struct CpuImage {
    pub width: u16,
    pub height: u16,
    pub rgba: Vec<u8>,
}
pub fn decode_image(path: &str, bytes: &[u8]) -> Result<CpuImage, ResourceError> {
    let image = image::load_from_memory(bytes)
        .map_err(|_| error(path, "图片解码失败"))?
        .into_rgba8();
    let width = u16::try_from(image.width()).map_err(|_| error(path, "图片尺寸超出后端限制"))?;
    let height = u16::try_from(image.height()).map_err(|_| error(path, "图片尺寸超出后端限制"))?;
    Ok(CpuImage {
        width,
        height,
        rgba: image.into_raw(),
    })
}
/// 单次装配所选向日葵 idle，采样借用源帧并保留源图层索引。
pub struct SunflowerIdle {
    animation: PreparedAnimation,
    segment: std::ops::Range<usize>,
    drawable_tracks: Vec<usize>,
}
impl SunflowerIdle {
    pub fn new(animation: PreparedAnimation) -> Result<Self, ResourceError> {
        let segment = animation
            .marker_segment("anim_idle")
            .ok_or_else(|| error(SUNFLOWER_ANIMATION, "没有向日葵 idle marker 区段"))?;
        let drawable_tracks = animation.drawable_tracks(&segment);
        if drawable_tracks.is_empty() {
            return Err(error(SUNFLOWER_ANIMATION, "向日葵 idle 没有可绘制轨道"));
        }
        Ok(Self {
            animation,
            segment,
            drawable_tracks,
        })
    }

    pub fn sample(
        &self,
        seconds: f64,
    ) -> impl Iterator<Item = (usize, &pvz_assets::AnimationFrame)> {
        let index = self.animation.frame_at(seconds, &self.segment);
        self.drawable_tracks.iter().filter_map(move |&layer| {
            let frame = self.animation.tracks[layer].get(index)?;
            (!frame.f.is_some_and(|n| n < 0.0) && frame.i.as_ref().is_some_and(|id| !id.is_empty()))
                .then_some((layer, frame))
        })
    }
}
/// 单次装配普通僵尸 walk；源索引和展开帧借用属于开发采样合同。
pub struct ZombieWalk {
    animation: PreparedAnimation,
    segment: std::ops::Range<usize>,
    drawable_tracks: Vec<usize>,
}
impl ZombieWalk {
    pub fn new(animation: PreparedAnimation) -> Result<Self, ResourceError> {
        let segment = animation
            .marker_segment("anim_walk")
            .ok_or_else(|| error(ZOMBIE_ANIMATION, "没有普通僵尸 walk marker 区段"))?;
        if ZOMBIE_TRACKS
            .iter()
            .any(|name| !animation.names.iter().any(|n| n == name))
        {
            return Err(error(ZOMBIE_ANIMATION, "普通僵尸 walk 指定身体轨道不存在"));
        }
        let drawable_tracks = animation
            .drawable_tracks(&segment)
            .into_iter()
            .filter(|&layer| ZOMBIE_TRACKS.contains(&animation.names[layer].as_str()))
            .collect::<Vec<_>>();
        if drawable_tracks.is_empty() {
            return Err(error(ZOMBIE_ANIMATION, "普通僵尸 walk 没有可绘制身体轨道"));
        }
        Ok(Self {
            animation,
            segment,
            drawable_tracks,
        })
    }
    pub fn sample(
        &self,
        seconds: f64,
    ) -> impl Iterator<Item = (usize, &pvz_assets::AnimationFrame)> {
        let index = self.animation.frame_at(seconds, &self.segment);
        self.drawable_tracks.iter().filter_map(move |&layer| {
            let frame = self.animation.tracks[layer].get(index)?;
            (!frame.f.is_some_and(|n| n < 0.0) && frame.i.as_ref().is_some_and(|id| !id.is_empty()))
                .then_some((layer, frame))
        })
    }
}
pub struct LoadedResources {
    pub background: CpuImage,
    pub sunflower_idle: SunflowerIdle,
    pub zombie_walk: ZombieWalk,
    pub images: BTreeMap<String, CpuImage>,
    pub effect: Vec<u8>,
    pub music: Vec<u8>,
    pub music_info: AudioInfo,
    pub poses: BTreeMap<VisualKind, Vec<pvz_assets::AnimationFrame>>,
    pub effects: BTreeMap<SoundCue, Vec<u8>>,
}
impl LoadedResources {
    pub async fn load() -> Result<Self, ResourceError> {
        let provider = AssetProvider::open().await?;
        Self::from_provider(provider).await
    }
    async fn from_provider(provider: AssetProvider) -> Result<Self, ResourceError> {
        let background = decode_image(BACKGROUND, &provider.read(BACKGROUND).await?)?;
        if background.width < 1020 || background.height < 600 {
            return Err(error(BACKGROUND, "图片不满足开发裁剪范围"));
        }
        let bytes = provider.read(ANIMATION).await?;
        let source =
            std::str::from_utf8(&bytes).map_err(|_| error(ANIMATION, "动画不是 UTF-8 文本"))?;
        let clip = parse_reanim(source).map_err(|_| error(ANIMATION, "动画解析失败"))?;
        let mapping = resolve_images(&clip, provider.manifest())?;
        let mut images = BTreeMap::new();
        for (id, path) in mapping {
            images.insert(id, decode_image(&path, &provider.read(&path).await?)?);
        }
        let animation = PreparedAnimation::new(&clip);
        if animation.marker_segment(DEVELOPMENT_IDLE_MARKER).is_none() {
            return Err(error(ANIMATION, "没有开发 idle marker 区段"));
        }
        let mut poses = BTreeMap::new();
        poses.insert(VisualKind::Peashooter, selected_pose(&animation, 79, &[])?);
        let bytes = provider.read(SUNFLOWER_ANIMATION).await?;
        let source = std::str::from_utf8(&bytes)
            .map_err(|_| error(SUNFLOWER_ANIMATION, "动画不是 UTF-8 文本"))?;
        let clip = parse_reanim(source).map_err(|_| error(SUNFLOWER_ANIMATION, "动画解析失败"))?;
        // 完整 clip 的所有引用都须通过映射与清单验证，不只检查 idle 区段。
        let mapping = resolve_images(&clip, provider.manifest())
            .map_err(|failure| error(SUNFLOWER_ANIMATION, failure.reason))?;
        for (id, path) in mapping {
            if let std::collections::btree_map::Entry::Vacant(entry) = images.entry(id) {
                let bytes = provider
                    .read(&path)
                    .await
                    .map_err(|failure| error(SUNFLOWER_ANIMATION, failure.reason))?;
                entry.insert(
                    decode_image(&path, &bytes)
                        .map_err(|failure| error(SUNFLOWER_ANIMATION, failure.reason))?,
                );
            }
        }
        let sunflower_idle = SunflowerIdle::new(PreparedAnimation::new(&clip))?;
        let bytes = provider.read(ZOMBIE_ANIMATION).await?;
        let source = std::str::from_utf8(&bytes)
            .map_err(|_| error(ZOMBIE_ANIMATION, "动画不是 UTF-8 文本"))?;
        let clip = parse_reanim(source).map_err(|_| error(ZOMBIE_ANIMATION, "动画解析失败"))?;
        let mapping = resolve_images(&clip, provider.manifest())
            .map_err(|failure| error(ZOMBIE_ANIMATION, failure.reason))?;
        for (id, path) in mapping {
            if let std::collections::btree_map::Entry::Vacant(entry) = images.entry(id) {
                let bytes = provider
                    .read(&path)
                    .await
                    .map_err(|failure| error(ZOMBIE_ANIMATION, failure.reason))?;
                entry.insert(
                    decode_image(&path, &bytes)
                        .map_err(|failure| error(ZOMBIE_ANIMATION, failure.reason))?,
                );
            }
        }
        let zombie_walk = ZombieWalk::new(PreparedAnimation::new(&clip))?;
        for (kind, path, frame, tracks) in POSE_SOURCES {
            let bytes = provider.read(path).await?;
            let source =
                std::str::from_utf8(&bytes).map_err(|_| error(path, "动画不是 UTF-8 文本"))?;
            let clip = parse_reanim(source).map_err(|_| error(path, "动画解析失败"))?;
            let mapping = resolve_images(&clip, provider.manifest())?;
            for (id, path) in mapping {
                if let std::collections::btree_map::Entry::Vacant(entry) = images.entry(id) {
                    entry.insert(decode_image(&path, &provider.read(&path).await?)?);
                }
            }
            let prepared = PreparedAnimation::new(&clip);
            poses.insert(*kind, selected_pose(&prepared, *frame, tracks)?);
        }
        images.insert(
            "PROJECTILEPEA".to_owned(),
            decode_image(PEA_IMAGE, &provider.read(PEA_IMAGE).await?)?,
        );
        poses.insert(
            VisualKind::Pea,
            vec![pvz_assets::AnimationFrame {
                i: Some("PROJECTILEPEA".to_owned()),
                x: Some(-14.0),
                y: Some(-14.0),
                ..Default::default()
            }],
        );
        let mut effects = BTreeMap::new();
        for (cue, path) in SOUND_SOURCES {
            let bytes = provider.read(path).await?;
            validate_audio(&bytes).map_err(|_| error(path, "音效 OGG 解码失败"))?;
            effects.insert(*cue, bytes);
        }
        let effect = provider.read(EFFECT).await?;
        validate_audio(&effect).map_err(|_| error(EFFECT, "音效 OGG 解码失败"))?;
        let music = provider.read(MUSIC).await?;
        let music_info = validate_audio(&music).map_err(|_| error(MUSIC, "音乐 OGG 解码失败"))?;
        Ok(Self {
            background,
            sunflower_idle,
            zombie_walk,
            images,
            effect,
            music,
            music_info,
            poses,
            effects,
        })
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    static REPOSITORY_RESOURCES: OnceLock<LoadedResources> = OnceLock::new();

    fn repository_resources() -> &'static LoadedResources {
        REPOSITORY_RESOURCES.get_or_init(|| {
            bevy::tasks::futures::now_or_never(
                LoadedResources::from_provider(repository_provider()),
            )
            .expect("native读取无需异步等待")
            .unwrap()
        })
    }
    #[test]
    fn 缺失登记文件不泄露临时绝对路径() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join("assets")).unwrap();
        std::fs::write(
            temp.path().join("assets/manifest.json"),
            include_bytes!("../../../assets/manifest.json"),
        )
        .unwrap();
        let provider = AssetProvider {
            index: pvz_assets::AssetRoot::open(temp.path()).unwrap(),
        };
        let result = provider.native_bytes(BACKGROUND).unwrap_err();
        assert_eq!(result.path, BACKGROUND);
        assert!(result.to_string().contains("文件缺失"));
        assert!(
            !result
                .to_string()
                .contains(&temp.path().display().to_string())
        );
        let invalid = provider.native_bytes("C:/private/secret.png").unwrap_err();
        assert!(!invalid.to_string().contains("private"));
    }
    fn repository_provider() -> AssetProvider {
        AssetProvider {
            index: pvz_assets::AssetRoot::open(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
            )
            .unwrap(),
        }
    }
    #[test]
    fn 登记资源完整加载切片构图背景与ogg() {
        let resources = repository_resources();
        assert_eq!(resources.images.len(), IMAGE_MAPPING.len() + 1);
        assert_eq!(
            resources.poses.keys().copied().collect::<Vec<_>>(),
            [
                VisualKind::Peashooter,
                VisualKind::Pea,
                VisualKind::Sun,
                VisualKind::LawnMower,
            ]
        );
        assert_eq!(resources.sunflower_idle.animation.fps, 12.0);
        assert_eq!(resources.sunflower_idle.animation.tracks.len(), 29);
        assert_eq!(resources.sunflower_idle.segment, 4..29);
        for (seconds, expected_frame) in [(0.0, 4), (24.0 / 12.0, 28), (25.0 / 12.0, 4)] {
            assert!(resources.sunflower_idle.sample(seconds).next().is_some());
            assert_eq!(
                resources
                    .sunflower_idle
                    .animation
                    .frame_at(seconds, &resources.sunflower_idle.segment),
                expected_frame
            );
            for (layer, frame) in resources.sunflower_idle.sample(seconds) {
                assert!(std::ptr::eq(
                    frame,
                    &resources.sunflower_idle.animation.tracks[layer][expected_frame]
                ));
                assert!(resources.images.contains_key(frame.i.as_ref().unwrap()));
            }
        }
        assert_eq!(resources.effects.len(), 4);
        assert_eq!(
            (resources.background.width, resources.background.height),
            (1400, 600)
        );
        assert!(resources.music_info.samples > 0);
        assert!(
            resources.poses[&VisualKind::Peashooter]
                .iter()
                .any(|frame| frame.i.as_deref() == Some("IMAGE_REANIM_PEASHOOTER_STALK_BOTTOM"))
        );
    }
    #[test]
    fn 登记向日葵缓存结构统计() {
        let resources = repository_resources();
        let idle = &resources.sunflower_idle;
        let animation = &idle.animation;
        assert_eq!(animation.fps, 12.0);
        assert_eq!(animation.tracks.len(), 29);
        assert_eq!(idle.segment, 4..29);
        assert_eq!(animation.tracks.len(), animation.names.len());
        assert!(animation.tracks.len() <= animation.tracks.capacity());
        assert!(animation.names.len() <= animation.names.capacity());
        assert!(idle.drawable_tracks.len() <= idle.drawable_tracks.capacity());
        assert_eq!(
            idle.drawable_tracks,
            animation.drawable_tracks(&idle.segment)
        );

        let mut expanded_frames = 0;
        let mut image_bytes = 0;
        let mut font_bytes = 0;
        let mut text_bytes = 0;
        let tracks: Vec<_> = animation
            .tracks
            .iter()
            .enumerate()
            .map(|(layer, track)| {
                assert!(track.len() <= track.capacity());
                expanded_frames += track.len();
                for frame in track {
                    image_bytes += frame.i.as_ref().map_or(0, String::len);
                    font_bytes += frame.font.as_ref().map_or(0, String::len);
                    text_bytes += frame.text.as_ref().map_or(0, String::len);
                }
                serde_json::json!({
                    "源索引": layer,
                    "len": track.len(),
                    "capacity": track.capacity(),
                })
            })
            .collect();
        let frame_string_bytes = image_bytes + font_bytes + text_bytes;
        let name_bytes: usize = animation.names.iter().map(String::len).sum();
        assert_eq!(expanded_frames, animation.tracks.iter().flatten().count());
        assert_eq!(
            frame_string_bytes,
            animation
                .tracks
                .iter()
                .flatten()
                .flat_map(|frame| [&frame.i, &frame.font, &frame.text])
                .filter_map(Option::as_ref)
                .map(String::len)
                .sum::<usize>()
        );
        assert_eq!(
            name_bytes,
            animation.names.iter().flat_map(|name| name.bytes()).count()
        );
        assert_eq!(tracks.len(), animation.tracks.len());
        assert!(expanded_frames > 0);
        assert!(image_bytes > 0);
        assert!(name_bytes > 0);

        // 只统计本字段拥有的结构，不测分配次数、进程峰值或 GPU 内存。
        let statistics = serde_json::json!({
            "范围": "登记向日葵缓存结构统计",
            "fps": animation.fps,
            "区段": { "start": idle.segment.start, "end": idle.segment.end },
            "展开帧数": expanded_frames,
            "帧字符串字节量": {
                "i": image_bytes,
                "font": font_bytes,
                "text": text_bytes,
                "合计": frame_string_bytes,
            },
            "names字节量": name_bytes,
            "外层轨道Vec": {
                "len": animation.tracks.len(),
                "capacity": animation.tracks.capacity(),
            },
            "各轨道Vec": tracks,
            "名称Vec": {
                "len": animation.names.len(),
                "capacity": animation.names.capacity(),
            },
            "源索引Vec": {
                "len": idle.drawable_tracks.len(),
                "capacity": idle.drawable_tracks.capacity(),
            },
        });
        println!("向日葵缓存结构统计：{statistics}");
    }
    #[test]
    fn 登记普通僵尸walk缓存结构统计() {
        let resources = repository_resources();
        let idle = &resources.zombie_walk;
        let animation = &idle.animation;
        assert_eq!(animation.fps, 12.0);
        assert_eq!(animation.tracks.len(), 42);
        assert_eq!(idle.segment, 44..91);
        assert_eq!(animation.tracks.len(), animation.names.len());
        assert!(animation.tracks.len() <= animation.tracks.capacity());
        assert!(animation.names.len() <= animation.names.capacity());
        assert!(idle.drawable_tracks.len() <= idle.drawable_tracks.capacity());
        assert_eq!(
            idle.drawable_tracks,
            animation
                .drawable_tracks(&idle.segment)
                .into_iter()
                .filter(|&layer| ZOMBIE_TRACKS.contains(&animation.names[layer].as_str()))
                .collect::<Vec<_>>()
        );

        let mut expanded_frames = 0;
        let mut image_bytes = 0;
        let mut font_bytes = 0;
        let mut text_bytes = 0;
        let tracks: Vec<_> = animation
            .tracks
            .iter()
            .enumerate()
            .map(|(layer, track)| {
                assert!(track.len() <= track.capacity());
                expanded_frames += track.len();
                for frame in track {
                    image_bytes += frame.i.as_ref().map_or(0, String::len);
                    font_bytes += frame.font.as_ref().map_or(0, String::len);
                    text_bytes += frame.text.as_ref().map_or(0, String::len);
                }
                serde_json::json!({
                    "源索引": layer,
                    "len": track.len(),
                    "capacity": track.capacity(),
                })
            })
            .collect();
        let frame_string_bytes = image_bytes + font_bytes + text_bytes;
        let name_bytes: usize = animation.names.iter().map(String::len).sum();
        assert_eq!(expanded_frames, animation.tracks.iter().flatten().count());
        assert_eq!(
            frame_string_bytes,
            animation
                .tracks
                .iter()
                .flatten()
                .flat_map(|frame| [&frame.i, &frame.font, &frame.text])
                .filter_map(Option::as_ref)
                .map(String::len)
                .sum::<usize>()
        );
        assert_eq!(
            name_bytes,
            animation.names.iter().flat_map(|name| name.bytes()).count()
        );
        assert_eq!(tracks.len(), animation.tracks.len());
        assert_eq!(expanded_frames, 21168);
        assert!(animation.tracks.iter().all(|track| track.len() == 504));
        assert_eq!(
            idle.drawable_tracks,
            [
                12, 13, 14, 17, 18, 19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 35, 36, 38, 39
            ]
        );
        assert!(image_bytes > 0);
        assert!(name_bytes > 0);

        // 只统计本字段拥有的结构，不测分配次数、进程峰值或 GPU 内存。
        let statistics = serde_json::json!({
            "范围": "登记普通僵尸walk缓存结构统计",
            "fps": animation.fps,
            "区段": { "start": idle.segment.start, "end": idle.segment.end },
            "展开帧数": expanded_frames,
            "帧字符串字节量": {
                "i": image_bytes,
                "font": font_bytes,
                "text": text_bytes,
                "合计": frame_string_bytes,
            },
            "names字节量": name_bytes,
            "外层轨道Vec": {
                "len": animation.tracks.len(),
                "capacity": animation.tracks.capacity(),
            },
            "各轨道Vec": tracks,
            "名称Vec": {
                "len": animation.names.len(),
                "capacity": animation.names.capacity(),
            },
            "源索引Vec": {
                "len": idle.drawable_tracks.len(),
                "capacity": idle.drawable_tracks.capacity(),
            },
        });
        println!("普通僵尸walk缓存结构统计：{statistics}");
    }
    #[test]
    fn 损坏与未登记输入被明确拒绝且不会泄露输入() {
        let provider = repository_provider();
        let bytes = provider.native_bytes(EFFECT).unwrap();
        let mut corrupt = bytes.clone();
        corrupt[0] ^= 1;
        assert!(
            validated_bytes(provider.manifest(), EFFECT, corrupt)
                .unwrap_err()
                .reason
                .contains("SHA-256")
        );
        let result = validated_bytes(provider.manifest(), "C:/private/key", bytes).unwrap_err();
        assert!(!result.to_string().contains("private"));
        assert!(decode_image("image.png", b"original-invalid-image-fixture").is_err());
    }
    #[test]
    fn 未知动画图像引用不按命名猜测() {
        let clip = parse_reanim(
            "<fps>12</fps><track><name>原创</name><t><i>IMAGE_REANIM_UNKNOWN</i></t></track>",
        )
        .unwrap();
        assert!(
            resolve_images(&clip, repository_provider().manifest())
                .unwrap_err()
                .reason
                .contains("唯一登记")
        );
    }
    #[test]
    fn 向日葵图像未登记时装配错误归属向日葵动画() {
        let provider = repository_provider();
        let temp = tempfile::tempdir().unwrap();
        let mut manifest: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../assets/manifest.json")).unwrap();
        let records = manifest["files"].as_array_mut().unwrap();
        let previous_len = records.len();
        records.retain(|record| record["path"] != "reanim/SunFlower_head.png");
        assert_eq!(records.len() + 1, previous_len);
        let entry_count = records.len() - 1;
        // 临时清单的非包条目数仍须满足 Manifest 的结构合同。
        manifest["package"]["entry_count"] = serde_json::Value::from(entry_count);
        manifest["package"]["extraction_check"] =
            serde_json::Value::from("原创测试变体：移除一个向日葵图像登记项");
        std::fs::create_dir(temp.path().join("assets")).unwrap();
        std::fs::write(
            temp.path().join("assets/manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

        // 只复制到向日葵映射失败前所需的登记输入，测试目录互相隔离。
        let shooter =
            parse_reanim(std::str::from_utf8(&provider.native_bytes(ANIMATION).unwrap()).unwrap())
                .unwrap();
        let mut required: BTreeSet<_> = [BACKGROUND, ANIMATION, "reanim/SunFlower.reanim"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        required.extend(
            resolve_images(&shooter, provider.manifest())
                .unwrap()
                .into_values(),
        );
        for path in required {
            let destination = temp.path().join("assets/original").join(&path);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::write(destination, provider.native_bytes(&path).unwrap()).unwrap();
        }
        let provider = AssetProvider {
            index: pvz_assets::AssetRoot::open(temp.path()).unwrap(),
        };
        let result = bevy::tasks::futures::now_or_never(LoadedResources::from_provider(provider))
            .expect("native读取无需异步等待");
        let error = match result {
            Ok(_) => panic!("未登记向日葵图像应使装配失败"),
            Err(error) => error,
        };
        assert_eq!(error.path, "reanim/SunFlower.reanim");
        assert_eq!(error.reason, "图像引用没有唯一登记映射");
    }
    #[test]
    fn 普通僵尸图像未登记时装配错误归属僵尸动画() {
        let provider = repository_provider();
        let temp = tempfile::tempdir().unwrap();
        let mut manifest: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../assets/manifest.json")).unwrap();
        let records = manifest["files"].as_array_mut().unwrap();
        let previous_len = records.len();
        records.retain(|record| record["path"] != "reanim/Zombie_head.png");
        assert_eq!(records.len() + 1, previous_len);
        let entry_count = records.len() - 1;
        // 临时清单的非包条目数仍须满足 Manifest 的结构合同。
        manifest["package"]["entry_count"] = serde_json::Value::from(entry_count);
        manifest["package"]["extraction_check"] =
            serde_json::Value::from("原创测试变体：移除一个僵尸图像登记项");
        std::fs::create_dir(temp.path().join("assets")).unwrap();
        std::fs::write(
            temp.path().join("assets/manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

        // 只复制到僵尸映射失败前所需的登记输入，测试目录互相隔离。
        let shooter =
            parse_reanim(std::str::from_utf8(&provider.native_bytes(ANIMATION).unwrap()).unwrap())
                .unwrap();
        let mut required: BTreeSet<_> =
            [BACKGROUND, ANIMATION, SUNFLOWER_ANIMATION, ZOMBIE_ANIMATION]
                .into_iter()
                .map(str::to_owned)
                .collect();
        required.extend(
            resolve_images(&shooter, provider.manifest())
                .unwrap()
                .into_values(),
        );
        let sunflower = parse_reanim(
            std::str::from_utf8(&provider.native_bytes(SUNFLOWER_ANIMATION).unwrap()).unwrap(),
        )
        .unwrap();
        required.extend(
            resolve_images(&sunflower, provider.manifest())
                .unwrap()
                .into_values(),
        );
        for (_, path, _, _) in POSE_SOURCES {
            required.insert((*path).to_owned());
            let clip =
                parse_reanim(std::str::from_utf8(&provider.native_bytes(path).unwrap()).unwrap())
                    .unwrap();
            required.extend(
                resolve_images(&clip, provider.manifest())
                    .unwrap()
                    .into_values(),
            );
        }
        for path in required {
            let destination = temp.path().join("assets/original").join(&path);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::write(destination, provider.native_bytes(&path).unwrap()).unwrap();
        }
        let provider = AssetProvider {
            index: pvz_assets::AssetRoot::open(temp.path()).unwrap(),
        };
        let result = bevy::tasks::futures::now_or_never(LoadedResources::from_provider(provider))
            .expect("native读取无需异步等待");
        let error = match result {
            Ok(_) => panic!("未登记僵尸图像应使装配失败"),
            Err(error) => error,
        };
        assert_eq!(error.path, ZOMBIE_ANIMATION);
        assert_eq!(error.reason, "图像引用没有唯一登记映射");
    }
    #[test]
    fn 登记默认开发片段包含头部嘴和茎() {
        let provider = repository_provider();
        let bytes = provider.native_bytes(ANIMATION).unwrap();
        let clip = parse_reanim(std::str::from_utf8(&bytes).unwrap()).unwrap();
        let animation = PreparedAnimation::new(&clip);
        let range = animation.marker_segment(DEVELOPMENT_IDLE_MARKER).unwrap();
        assert_eq!(range, 79..104);
        for name in ["anim_face", "idle_mouth", "stalk_top"] {
            let index = animation.names.iter().position(|n| n == name).unwrap();
            for frame in &animation.tracks[index][range.clone()] {
                assert!(
                    frame.f.is_some_and(|f| f >= 0.0),
                    "{name} 必须在默认开发片段中可见"
                );
                assert!(frame.i.as_ref().is_some_and(|id| !id.is_empty()));
            }
        }
    }

    #[test]
    fn 登记普通僵尸walk整段选择身体排除附件并借用源帧() {
        let resources = repository_resources();
        let walk = &resources.zombie_walk;
        let layers = [
            12, 13, 14, 17, 18, 19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 35, 36, 38, 39,
        ];
        let body_ids: BTreeSet<_> = layers
            .iter()
            .map(|&layer| walk.animation.tracks[layer][44].i.as_deref().unwrap())
            .collect();
        assert_eq!(body_ids.len(), 19);
        for step in 0..47 {
            let frames: Vec<_> = walk.sample(f64::from(step) / 12.0).collect();
            assert_eq!(
                frames.iter().map(|(layer, _)| *layer).collect::<Vec<_>>(),
                layers
            );
            for (layer, frame) in frames {
                assert!(std::ptr::eq(
                    frame,
                    &walk.animation.tracks[layer][44 + step as usize]
                ));
                assert_eq!(
                    frame.a.unwrap_or(1.0),
                    1.0,
                    "资源缺省 alpha 的有效绘制值为 1"
                );
                let id = frame.i.as_deref().unwrap();
                assert!(body_ids.contains(id));
                assert!(resources.images.contains_key(id));
                assert!(
                    ![
                        "BUCKET",
                        "CONE",
                        "SCREENDOOR",
                        "DUCKYTUBE",
                        "MUSTACHE",
                        "FLAGHAND"
                    ]
                    .iter()
                    .any(|part| id.contains(part))
                );
            }
        }
        assert_eq!(walk.animation.frame_at(47.0 / 12.0, &walk.segment), 44);
        assert!(!resources.poses.contains_key(&VisualKind::Zombie));
        for pose in resources.poses.values() {
            assert!(!pose.is_empty());
            assert!(
                pose.iter()
                    .all(|f| resources.images.contains_key(f.i.as_ref().unwrap()))
            );
        }
    }
}
