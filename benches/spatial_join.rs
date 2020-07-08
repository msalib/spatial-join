use criterion::{criterion_group, criterion_main, Criterion};
use geo::{Coordinate, Line};
use rand;
use rand::Rng;
use wkt::ToWkt;

#[cfg(feature = "parallel")]
use rayon::iter::ParallelIterator;

extern crate spatial_join;

fn buffer(source: &geo::Geometry<f64>, width: f64, quadsegs: i32) -> geo::Polygon<f64> {
    // This is going to be gross because we're going to call out to
    // libgeos, but the only way to do that is by going through WKT
    // twice.
    let source_wkt = source.to_wkt();
    assert_eq!(source_wkt.items.len(), 1);
    let source_wkt_str = source_wkt.items[0].to_string();

    let intermediate = geos::Geometry::new_from_wkt(&source_wkt_str).unwrap();
    let geos_result = intermediate.buffer(width, quadsegs).unwrap();
    let geos_wkt_str = geos_result.to_wkt().unwrap();

    let dest_wkt = wkt::Wkt::from_str(&geos_wkt_str).unwrap();
    assert_eq!(dest_wkt.items.len(), 1);
    let dest_geo = wkt::conversion::try_into_geometry(&dest_wkt.items[0]).unwrap();
    match dest_geo {
        geo::Geometry::Polygon(poly) => poly,
        _ => panic!(),
    }
}

/// This creates `n` random lines of length at most `max_len` and then
/// buffers them by `buffer_width` in a space that is `height` by
/// `width`. The result is lots of pill-boxes (long rectangles with
/// half circles at either end).
fn generate_polys(
    n: usize,
    max_len: f64,
    buffer_width: f64,
    width: f64,
    height: f64,
) -> Vec<geo::Geometry<f64>> {
    let mut rng = rand::thread_rng();
    let mut result = Vec::with_capacity(n);
    for _ in 0..n {
        let x0 = rng.gen::<f64>() * width;
        let y0 = rng.gen::<f64>() * height;
        let angle_rad = rng.gen::<f64>() * std::f64::consts::PI * 2.0;
        let (sin, cos) = angle_rad.sin_cos();
        let x1 = x0 + (max_len * cos);
        let y1 = y0 + (max_len * sin);

        let line = Line::new(Coordinate { x: x0, y: y0 }, Coordinate { x: x1, y: y1 });
        let poly = buffer(&geo::Geometry::Line(line), buffer_width, 16);
        result.push(poly.into());
    }
    result
}

// Sigh. Our workflow for posting benchmarks assumes that bench names
// have no spaces:
// https://github.com/pksunkara/github-action-benchmark/blob/master/src/extract.ts#L192

fn serial_benchmark(c: &mut Criterion) {
    let polys1k = generate_polys(1_000, 40., 5., 5_000., 2_000.);

    c.bench_function("1k_load", |b| {
        b.iter(|| {
            spatial_join::Config::new().serial(&polys1k).unwrap();
        })
    });

    c.bench_function("1k_self_spatial_join", |b| {
        b.iter(|| {
            let si = spatial_join::Config::new().serial(&polys1k).unwrap();
            let v: Vec<_> = si
                .spatial_join(&polys1k, spatial_join::Interaction::Intersects)
                .unwrap()
                .collect();
            v
        })
    });
}

#[cfg(feature = "parallel")]
fn parallel_benchmark(c: &mut Criterion) {
    let polys5k = generate_polys(5_000, 40., 5., 5_000., 2_000.);

    c.bench_function("5k self spatial join", |b| {
        b.iter(|| {
            let si = spatial_join::Config::new().serial(&polys5k).unwrap();
            let v: Vec<_> = si
                .spatial_join(&polys5k, spatial_join::Interaction::Intersects)
                .unwrap()
                .collect();
            v
        })
    });

    c.bench_function("5k self par spatial join", |b| {
        b.iter(|| {
            let si = spatial_join::Config::new().parallel(&polys5k).unwrap();
            let v: Vec<_> = si
                .spatial_join(&polys5k, spatial_join::Interaction::Intersects)
                .unwrap()
                .collect();
            v
        })
    });
}

#[cfg(not(feature = "parallel"))]
criterion_group!(benches, serial_benchmark);
#[cfg(feature = "parallel")]
criterion_group!(benches, serial_benchmark, parallel_benchmark);
criterion_main!(benches);
