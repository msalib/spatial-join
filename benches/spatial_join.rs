use criterion::{criterion_group, criterion_main, Criterion};
extern crate spatial_join;

fn load(path: &str) -> Vec<geo::Geometry<f64>> {
    let wkt_str = std::fs::read_to_string(path);
    let mut geos: Vec<geo::Geometry<f64>> = Vec::new();
    let w: wkt::Wkt<f64> = wkt::Wkt::from_str(&wkt_str.unwrap()).unwrap();
    for wg in w.items {
        let g = wkt::conversion::try_into_geometry(&wg).unwrap();
        if let geo::Geometry::GeometryCollection(gc) = g {
            geos.extend(gc.into_iter());
        } else {
            geos.push(g);
        }
    }
    geos
}

fn criterion_benchmark(c: &mut Criterion) {
    let polys1k = load("polys1k.wkt");
    // let polys10k = load("polys10k.wkt");
    c.bench_function("1k load", |b| {
        b.iter(|| {
            spatial_join::Config::new().serial(&polys1k).unwrap();
        })
    });

    c.bench_function("1k self spatial join", |b| {
        b.iter(|| {
            let si = spatial_join::Config::new().serial(&polys1k).unwrap();
            let v: Vec<_> = si
                .spatial_join(&polys1k, spatial_join::Interaction::Intersects)
                .unwrap()
                .collect();
            v
        })
    });

    // c.bench_function("10k self spatial join", |b| {
    //     b.iter(|| {
    //         let si = spatial_join::Config::new().serial(&polys10k).unwrap();
    //         let v: Vec<_> = si
    //             .spatial_join(&polys10k, spatial_join::Interaction::Intersects)
    //             .unwrap()
    //             .collect();
    //         v
    //     })
    // });

    // c.bench_function("10k self par spatial join", |b| {
    //     b.iter(|| {
    //         use rayon::iter::ParallelIterator;

    //         let si = spatial_join::Config::new().parallel(&polys10k).unwrap();
    //         let v: Vec<_> = si
    //             .spatial_join(&polys10k, spatial_join::Interaction::Intersects)
    //             .unwrap()
    //             .collect();
    //         v
    //     })
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
