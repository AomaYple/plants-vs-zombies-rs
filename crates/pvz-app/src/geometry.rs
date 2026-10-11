use crate::animation::Affine;

/// 与 GPU 无关的顶点；裁剪时插值 UV，避免裁剪改变纹理形状。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteVertex {
    pub point: [f32; 2],
    pub uv: [f32; 2],
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipRect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}
/// 仿射四边形经凸多边形裁剪后保持源顶点顺序，供表现层逐图层提交。
pub fn clipped_quad(transform: Affine, size: [f32; 2], clip: ClipRect) -> Vec<SpriteVertex> {
    if clip
        .min
        .iter()
        .chain(clip.max.iter())
        .any(|n| !n.is_finite())
        || clip.min[0] >= clip.max[0]
        || clip.min[1] >= clip.max[1]
    {
        return Vec::new();
    }
    let mut vertices = [
        [0.0, 0.0],
        [size[0], 0.0],
        [size[0], size[1]],
        [0.0, size[1]],
    ]
    .into_iter()
    .zip([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
    .map(|(point, uv)| SpriteVertex {
        point: transform.transform(point),
        uv,
    })
    .collect::<Vec<_>>();
    if vertices
        .iter()
        .flat_map(|v| v.point)
        .any(|n| !n.is_finite())
    {
        return Vec::new();
    }
    for (axis, bound, greater) in [
        (0, clip.min[0], true),
        (0, clip.max[0], false),
        (1, clip.min[1], true),
        (1, clip.max[1], false),
    ] {
        let input = std::mem::take(&mut vertices);
        let Some(mut previous) = input.last().copied() else {
            break;
        };
        let inside = |v: SpriteVertex| {
            if greater {
                v.point[axis] >= bound
            } else {
                v.point[axis] <= bound
            }
        };
        for current in input {
            if inside(previous) != inside(current) {
                let t =
                    (bound - previous.point[axis]) / (current.point[axis] - previous.point[axis]);
                let interpolate =
                    |a: [f32; 2], b: [f32; 2]| [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])];
                vertices.push(SpriteVertex {
                    point: interpolate(previous.point, current.point),
                    uv: interpolate(previous.uv, current.uv),
                });
            }
            if inside(current) {
                vertices.push(current);
            }
            previous = current;
        }
    }
    vertices
}
