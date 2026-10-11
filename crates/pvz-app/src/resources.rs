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
pub const EFFECT: &str = "sounds/seedlift.ogg";
pub const MUSIC: &str = "sounds/ZombiesOnYourLawn.ogg";
/// 开发默认播放片段；必须用登记 XML 确认包含整个植物。
pub const DEVELOPMENT_IDLE_MARKER: &str = "anim_full_idle";
/// 已逐项核对的开发映射；resources.xml 没有登记 PeaShooter 图像。
pub const IMAGE_MAPPING: &[(&str, &str)] = &[
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
pub struct LoadedResources {
    pub background: CpuImage,
    pub animation: PreparedAnimation,
    pub images: BTreeMap<String, CpuImage>,
    pub effect: Vec<u8>,
    pub music: Vec<u8>,
    pub music_info: AudioInfo,
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
        let effect = provider.read(EFFECT).await?;
        validate_audio(&effect).map_err(|_| error(EFFECT, "音效 OGG 解码失败"))?;
        let music = provider.read(MUSIC).await?;
        let music_info = validate_audio(&music).map_err(|_| error(MUSIC, "音乐 OGG 解码失败"))?;
        Ok(Self {
            background,
            animation,
            images,
            effect,
            music,
            music_info,
        })
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
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
    fn 登记资源完整加载19图像背景与ogg() {
        let resources = bevy::tasks::futures::now_or_never(LoadedResources::from_provider(
            repository_provider(),
        ))
        .expect("native读取无需异步等待")
        .unwrap();
        assert_eq!(resources.images.len(), 19);
        assert_eq!(
            (resources.background.width, resources.background.height),
            (1400, 600)
        );
        assert!(resources.music_info.samples > 0);
        assert!(resources.animation.marker_segment("anim_idle").is_some());
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
}
