use std::convert::TryInto;

use geo::{Coordinate, Geometry, Line, LineString, Point, Polygon, Rect, Triangle};
use proptest::prelude::*;

use crate::relates::Relates;
use crate::{Config, Interaction, SplitGeoSeq};

use super::naive::{slow_prox_map, slow_spatial_join};

#[rustfmt::skip]
prop_compose! {
    fn arb_point()(x in -1.0..1.0, y in -1.0..1.0) -> Point<f64> {
	Point::new(x, y)
    }
}

#[rustfmt::skip]
prop_compose! {
    fn arb_line()(start in arb_point(), end in arb_point()) -> Line<f64> {
	Line::new(start, end)
    }
}

#[rustfmt::skip]
prop_compose! {
    fn arb_linestring()(points in prop::collection::vec(arb_point(), 2..20)) -> LineString<f64> {
	points.into()
    }
}

#[rustfmt::skip]
prop_compose! {
    fn arb_rect()(center in arb_point(),
		  width in 0.0..1.0, height in 0.0..1.0) -> Rect<f64> {
	let min: Coordinate<f64> = (
	    center.x() - width / 2.,
	    center.y() - height / 2.).into();
	let max: Coordinate<f64> = (
	    center.x() + width / 2.,
	    center.y() + height/2.).into();
	Rect::new(min, max)
    }
}

#[rustfmt::skip]
prop_compose! {
    fn arb_poly()(center in arb_point(),
		  exterior_points in 3..17,
		  radius in 0.000001..0.5) -> Polygon<f64> {
	let angles = (0..exterior_points)
	    .map(|idx| 2.0 * std::f64::consts::PI * (idx as f64) / (exterior_points as f64));
	let points: Vec<geo::Coordinate<f64>> = angles
	    .map(|angle_rad| angle_rad.sin_cos())
	    .map(|(sin, cos)| geo::Coordinate {
		x: center.x() + radius * cos,
		y: center.y() + radius * sin,
	    })
	    .collect();

	Polygon::new(geo::LineString(points), vec![])
    }
}

#[rustfmt::skip]
prop_compose! {
    fn arb_triangle()(a in arb_point(),
		      b in arb_point(),
		      c in arb_point()) -> Triangle<f64> {
	Triangle(a.0, b.0, c.0)
    }
}

fn geo_strat() -> impl Strategy<Value = Geometry<f64>> {
    prop_oneof![
        arb_point().prop_map(Geometry::Point),
        arb_line().prop_map(Geometry::Line),
        arb_poly().prop_map(Geometry::Polygon),
        arb_linestring().prop_map(Geometry::LineString),
        arb_rect().prop_map(Geometry::Rect),
        arb_triangle().prop_map(Geometry::Triangle)
    ]
}

#[rustfmt::skip]
prop_compose! {
    fn arb_splitgeoseq()(
	geos in prop::collection::vec(geo_strat(), 0..100)) -> SplitGeoSeq {
	(&geos).try_into().unwrap()
    }
}

#[rustfmt::skip]
proptest! {
    #[test]
    fn prox_map_vs_slow(
	  small in arb_splitgeoseq(),
	  big in arb_splitgeoseq(),
	  max_distance in 0.0..4.0) {
	use crate::tests::test_prox_map;
	let expected = slow_prox_map(&small, &big, max_distance);
	// This tests both .proximity_map and .proximity_map_with_geos
	// and their parallel variants when available.
	test_prox_map(Config::new().max_distance(max_distance), small, big, expected);
    }
}

fn interaction_strat() -> impl Strategy<Value = Interaction> {
    prop_oneof![
        Just(Interaction::Intersects),
        Just(Interaction::Within),
        Just(Interaction::Contains),
    ]
}

#[rustfmt::skip]
proptest! {
    #[test]
    fn spatial_join_vs_slow(
	  small in arb_splitgeoseq(),
	  big in arb_splitgeoseq(),
	  interaction in interaction_strat()) {
	use crate::tests::test_spatial_join;
	let expected = slow_spatial_join(&small, &big, interaction);
	// This tests both .spatial_join and .spatial_join_with_geos
	// and their parallel variants when available.
	test_spatial_join(Config::new(), small, big, interaction, expected);
    }
}

#[rustfmt::skip]
proptest! {
    #[test]
    fn compare_relates_to_libgeos(
  	  a in geo_strat(),
	  b in geo_strat()) {

	fn wkt_str(x: &Geometry<f64>) -> String {
	    use wkt::ToWkt;
	    let w = x.to_wkt();
	    assert_eq!(w.items.len(), 1);
	    w.items[0].to_string()
	}

	// FIXME: find a way to use lifetime annotations to convince
	// rustc that the return value doesn't use the borrow at all
	fn convert(x: &Geometry<f64>) -> geos::Geometry {
	    geos::Geometry::new_from_wkt(&wkt_str(x)).unwrap()
	}

	let geos_a = convert(&a);
	let geos_b = convert(&b);

	let a2 = a.clone();
	let b2 = b.clone();
	assert_eq!(
	    crate::enum_dispatch!(a2, b2, a2.Intersects(&b2)),
	    geos_a.intersects(&geos_b).unwrap());

	let a2 = a.clone();
	let b2 = b.clone();
	assert_eq!(
	    crate::enum_dispatch!(a2, b2, a2.Contains(&b2)),
	    geos_a.contains(&geos_b).unwrap());

	// I don't want to test distance checks for stuff that we
	// didn't write because I keep finding issues that I don't
	// feel like writing up.
	if match &a {Geometry::Rect(_) | Geometry::Triangle(_) => true, _ => false} {
	    let a2 = a.clone();
	    let b2 = b.clone();
	    let relates_dist = crate::enum_dispatch!(a2, b2, a2.EuclideanDistance(&b2));
	    let geos_dist = geos_a.distance(&geos_b).unwrap();
	    if !approx::relative_eq!(relates_dist, geos_dist, max_relative = 0.2) {
		let a2 = a.clone();
		let b2 = b.clone();
		prop_assert_eq!(
		    relates_dist, geos_dist,
		    "distance fail relates: {} != geos: {} \nfor gpd.GeoSeries([loads('{}'), loads('{}')]).plot()\n{:#?}\n{:#?}\n",
		    relates_dist, geos_dist, wkt_str(&a2), wkt_str(&b2), a2.clone(), b2.clone());
	    }
	}
    }
}
