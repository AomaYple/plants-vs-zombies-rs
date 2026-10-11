use criterion::{Criterion, criterion_group, criterion_main};
use pvz_assets::{parse_particle, parse_reanim};
use std::hint::black_box;

const REANIM: &str =
    "<root><fps>12</fps><track><name>idle</name><t><f>0</f></t><t></t></track></root>";
const PARTICLE: &str = "<Emitter><Name>原创</Name><EmitterType>Box</EmitterType><Image>IMAGE_TEST</Image><SystemDuration>100</SystemDuration><Field><FieldType>Position</FieldType></Field></Emitter>";
const SPARSE: &str = include_str!("../../../tests/fixtures/assets-minimal/sparse-frames.reanim");
const MULTIPLE: &str = include_str!("../../../tests/fixtures/assets-minimal/multiple-emitters.xml");

fn 解析原创夹具(c: &mut Criterion) {
    c.bench_function(
        &format!("Reanim稀疏字段合同-1轨道2帧-{}字节", REANIM.len()),
        |bencher| {
            bencher.iter(|| black_box(parse_reanim(black_box(REANIM)).unwrap()));
        },
    );
    c.bench_function(
        &format!("粒子属性树合同-1发射器1Field-{}字节", PARTICLE.len()),
        |bencher| {
            bencher.iter(|| black_box(parse_particle(black_box(PARTICLE)).unwrap()));
        },
    );
    c.bench_function(
        &format!("Reanim稀疏字段合同-1轨道2帧全字段-{}字节", SPARSE.len()),
        |bencher| {
            bencher.iter(|| black_box(parse_reanim(black_box(SPARSE)).unwrap()));
        },
    );
    c.bench_function(
        &format!(
            "粒子属性树合同-2发射器1Field1SystemField-{}字节",
            MULTIPLE.len()
        ),
        |bencher| {
            bencher.iter(|| black_box(parse_particle(black_box(MULTIPLE)).unwrap()));
        },
    );
}

criterion_group!(benches, 解析原创夹具);
criterion_main!(benches);
