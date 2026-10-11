use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pvz_assets::{parse_particle, parse_reanim};

const REANIM: &str =
    "<root><fps>12</fps><track><name>idle</name><t><f>0</f></t><t></t></track></root>";
const PARTICLE: &str = "<Emitter><Name>原创</Name><EmitterType>Box</EmitterType><Image>IMAGE_TEST</Image><SystemDuration>100</SystemDuration><Field><FieldType>Position</FieldType></Field></Emitter>";

fn 解析原创夹具(c: &mut Criterion) {
    c.bench_function("Reanim 最小夹具解析", |bencher| {
        bencher.iter(|| black_box(parse_reanim(black_box(REANIM)).unwrap()));
    });
    c.bench_function("粒子 XML 最小夹具解析", |bencher| {
        bencher.iter(|| black_box(parse_particle(black_box(PARTICLE)).unwrap()));
    });
}

criterion_group!(benches, 解析原创夹具);
criterion_main!(benches);
