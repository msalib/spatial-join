//! `spatial-join` provides tools to perform streaming geospatial-joins on geographic data.
//!
//! ## Spatial Joins
//!
//! Given two sequences of geospatial shapes, `small` and `big`, a
//! spatial-join indicates which elements of `small` and `big`
//! intersect. You could compute this yourself using a nested loop,
//! but like any good spatial-join package, this one uses
//! [R-trees](https://en.wikipedia.org/wiki/R-tree) to dramatically
//! reduce the search space.
//!
//! We're not limited to intersections only! We can also find pairs
//! where elements of `small` contain elements of `big` or are within
//! elements of `big` by passing different values of
//! [Interaction](./enum.Interaction.html).

//! ## Proximity Maps
//!
//! While spatial join is a well known term, proximity map is
//! not. Given two sequences of shapes `small` and `big`, it just
//! finds all pairs of items whose distance is less than some
//! threshold. You set that threshold using the
//! [`max_distance`](./struct.Config.html#method.max_distance) method
//! on the [`Config`](./struct.Config.html) struct.
//!
//! ## Inputs
//!
//! Inputs are sequences of shapes, and shapes must be one of the
//! following elements from the
//! [`geo`](https://docs.rs/geo/latest/geo/) crate:
//! * [points](https://docs.rs/geo/latest/geo/struct.Point.html),
//! * [lines](https://docs.rs/geo/latest/geo/struct.Line.html),
//! * [line strings](https://docs.rs/geo/latest/geo/struct.LineString.html),
//! * [polygons](https://docs.rs/geo/latest/geo/struct.Polygon.html),
//! * [rectangles](https://docs.rs/geo/latest/geo/struct.Rect.html),
//! * [triangles](https://docs.rs/geo/latest/geo/struct.Triangle.html), or
//! * the [Geometry](https://docs.rs/geo/latest/geo/enum.Geometry.html) enum
//!
//! `MultiPoint`, `MultiLineString`, and `MultiPolygon` are *not* supported.
//!
//! While the [geo] crate makes these types generic over the
//! coordinate type, `spatial-join` only supports [geo] types
//! parametrized with [std::f64] coordinate types (i.e.,
//! `Polygon<f64>`).
//!
//! So what kind of sequences can you use?
//! * slices: `&[T]`,
//! * vectors: `Vec<T>` or `&Vec<T>`, or
//! * [`&geo::GeometryCollection`](https://docs.rs/geo/latest/geo/struct.GeometryCollection.html)
//!
//! In addition:
//! * all coordinate values must be finite
//! * `LineStrings` must have at least two points
//! * `Polygon` exteriors must have at least three points
//!
//! Input that doesn't meet these conditions will return an [error](./enum.Error.html).
//!
//! ## Outputs
//!
//! [`SpatialIndex::spatial_join`](./struct.SpatialIndex.html#method.spatial_join) returns `Result<impl
//! Iterator<Item=SJoinRow>, Error>` where
//! [`SJoinRow`](./struct.SJoinRow.html) gives you indexes into
//! `small` and `big` to find the corresponding geometries.
//!
//! Alternatively, you can use [`SpatialIndex::spatial_join_with_geos`](./struct.SpatialIndex.html#method.spatial_join_with_geos)
//! which returns `Result<impl Iterator<Item=SJoinGeoRow>, Error>`.
//! [`SJoinGeoRow`](./struct.SJoinGeoRow.html) differs from
//! [`SJoinRow`](./struct.SJoinRow.html) only in the addition of `big`
//! and `small`
//! [`Geometry`](https://docs.rs/geo/latest/geo/enum.Geometry.html)
//! fields so you can work directly with the source geometries without
//! having to keep the original sequences around. This convenience
//! comes at the cost of cloning the source geometries which can be
//! expensive for geometries that use heap storage like `LineString`
//! and `Polygon`.
//!
//! In a similar manner, [`SpatialIndex::proximity_map`](./struct.SpatialIndex.html#method.proximity_map) and
//! [`SpatialIndex::proximity_map_with_geos`](./struct.SpatialIndex.html#method.proximity_map) offer
//! [`ProxMapRow`](./struct.ProxMapRow.html) and
//! [`ProxMapGeoRow`](./struct.ProxMapGeoRow.html) iterators in their
//! return types. These differ from their `SJoin` counterparts only in
//! the addition of a `distance` field.
//!
//! ## Examples
//!
//! Here's the simplest thing: let's verify that a point intersects itself.
//! ```
//! use spatial_join::*;
//! use geo::{Geometry, Point};
//! fn foo() -> Result<(), Error> {
//!     // Create a new spatial index loaded with just one point
//!     let idx = Config::new()
//!         // Ask for a serial index that will process data on only one core
//!         .serial(vec![Geometry::Point(Point::new(1.1, 2.2))])?;
//!     let results: Vec<_> = idx
//!         .spatial_join(
//!             vec![Geometry::Point(Point::new(1.1, 2.2))],
//!             Interaction::Intersects,
//!         )?
//!         .collect(); // we actually get an iterator, but let's collect it into a Vector.
//!     assert_eq!(
//!         results,
//!         vec![SJoinRow {
//!             big_index: 0,
//!             small_index: 0
//!         }]);
//!     Ok(())
//! }
//! foo();
//! ```
//!
//! For a slightly more complicated, we'll take a box and a smaller
//! box and verify that the big box contains the smaller box, and
//! we'll do it all in parallel.
//! ```
//! #[cfg(feature = "parallel")] {
//!     use spatial_join::*;
//!     use geo::{Coordinate, Geometry, Point, Rect};
//!     use rayon::prelude::*;
//!
//!     fn bar() -> Result<(), Error> {
//!         let idx = Config::new()
//!              .parallel(vec![Geometry::Rect(Rect::new(
//!                  Coordinate { x: -1., y: -1. },
//!                  Coordinate { x: 1., y: 1. },
//!              ))])?;
//!          let results: Vec<_> = idx
//!              .spatial_join(
//!                  vec![Geometry::Rect(Rect::new(
//!                      Coordinate { x: -0.5, y: -0.5 },
//!                      Coordinate { x: 0.5, y: 0.5 },
//!              ))],
//!                  Interaction::Contains,
//!              )?
//!              .collect();
//!          assert_eq!(
//!              results,
//!              vec![SJoinRow {
//!                  big_index: 0,
//!                  small_index: 0
//!              }]
//!          );
//!          Ok(())
//!     }
//!     bar();
//! }
//! ```
//!
//! ## Crate Features
//!
//! - `parallel`
//!   - Enabled by default.
//!   - This adds a dependency on
//!     [`rayon`](https://crates.io/crates/rayon) and provides a
//!     [`parallel`](./struct.Config.html#method.parallel) method that
//!     returns a [`ParSpatialIndex`](./struct.ParSpatialIndex.html)
//!     just like the [`SpatialIndex`](./struct.SpatialIndex.html)
//!     that [`serial`](./struct.Config.html#method.serial) returns
//!     except that all the methods return `Result<impl
//!     ParallelIterator>` instead of `Result<impl Iterator>`.
//!
//! ## Geographic
//!
//! Right now, this entire crate assumes that you're dealing with
//! euclidean geometry on a two-dimensional plane. But that's unusual:
//! typically you've got geographic coordinates (longitude and
//! latitude measured in decimal degrees). To use the tools in this
//! package correctly, you should really reproject your geometries
//! into an appropriate euclidean coordinate system. That might be
//! require you to do a lot of extra work if the extent of your
//! geometry sets exceeds what any reasonable projection can handle.
//!
//! Alternatively, you can just pretend that geodetic coordinates are
//! euclidean. For spatial-joins that will mostly work if all of your
//! geometries steer well-clear of the anti-meridian (longitude=±180
//! degrees) and the polar regions as well.
//!
//! For proximity maps, you'll need to pick an appropriate
//! `max_distance` value measured in decimal degrees which will be
//! used for both longitude and latitude offsets
//! simulataneously. That's challenging because while one degree of
//! latitude is always the same (about 110 km), one degree of
//! longitude changes from about 110 km at the equator to 0 km at the
//! poles. If your geometry sets have a narrow extant and are near the
//! equator, you might be able to find a `max_distance` value that
//! works, but that's pretty unlikely.
//!
//! ## Performance
//!
//! * You'll notice that our API specifies geometry sequences in terms
//!   of `small` and `big`. In order to construct a spatial index
//!   object, we have to build a series of R-trees, one per geometry
//!   type, using bulk loading. This process is expensive
//!   (`O(n*log(n))`) so you'll probably get better overall performance
//!   if you index the smaller sequence.
//! * Because the spatial-join and proximity-map operations are
//!   implemented as iterators, you can process very large data-sets
//!   with low memory usage. But you do need to keep both the `small`
//!   and `large` geometry sequence in memory, in addition to rtrees
//!   for the `small` sequence. Note that in some cases, specifically
//!   whenever we're processing a heap-bound element of the `large`
//!   sequence (i.e., Polygons or LineStrings), we will buffer all
//!   matching result records for each such `large` geometry.
//! * If you use a non-zero `max_distance` value, then any
//!   spatial-join operations will be somewhat slower since
//!   `max_distance` effectively buffers `small` geometries in the
//!   r-trees. You'll still get the correct answer, but it might take
//!   longer. The larger the `max_distance` value, the longer it will
//!   take.
//!
//! ## License
//!
//! Licensed under either of
//!
//!  * Apache License, Version 2.0
//!    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
//!  * MIT license
//!    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
//!
//! at your option.
//!
//! ## Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
//! dual licensed as above, without any additional terms or conditions.
//!

use rstar::RTree;

mod structs;
pub use structs::*;

mod validation;

mod conv;

mod relates;

mod rtrees;
use rtrees::FakeRegion;

#[derive(Debug)]
pub struct SpatialIndex {
    small: SplitGeoSeq,
    point_tree: RTree<FakeRegion>,
    line_tree: RTree<FakeRegion>,
    poly_tree: RTree<FakeRegion>,
    ls_tree: RTree<FakeRegion>,
    rect_tree: RTree<FakeRegion>,
    tri_tree: RTree<FakeRegion>,
    config: Config,
}

#[cfg(feature = "parallel")]
pub struct ParSpatialIndex(SpatialIndex);

mod index;

#[cfg(test)]
mod naive;

#[cfg(test)]
mod proptests;

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use geo::Point;
    use pretty_assertions::assert_eq;

    #[cfg(feature = "parallel")]
    use rayon::prelude::*;

    use super::*;
    use index::*;

    pub fn test_prox_map<Small, Big, E1, E2>(
        config: Config,
        small: Small,
        big: Big,
        expected: &Vec<ProxMapRow>,
    ) where
        Small: TryInto<SplitGeoSeq, Error = E1> + Clone,
        Big: TryInto<SplitGeoSeq, Error = E2> + Clone,
        E1: std::any::Any + std::fmt::Debug,
        E2: std::any::Any + std::fmt::Debug,
    {
        //assert!(expected.is_sorted());
        let small_geoms = sgs_try_into(small.clone())
            .expect("small conversion")
            .to_vec();
        let big_geoms = sgs_try_into(big.clone()).expect("big conversion").to_vec();
        let expected_geoms: Vec<_> = expected
            .iter()
            .map(|pmr| ProxMapGeoRow {
                big_index: pmr.big_index,
                small_index: pmr.small_index,
                distance: pmr.distance,
                big: big_geoms[pmr.big_index].clone(),
                small: small_geoms[pmr.small_index].clone(),
            })
            .collect();
        let _expected_geoms2 = expected_geoms.clone();

        let si = config
            .clone()
            .serial(small.clone())
            .expect("construction succeeded");
        let mut actual = si.proximity_map(big.clone()).unwrap().collect::<Vec<_>>();
        actual.sort();
        assert_eq!(actual, *expected);

        let mut actual_geoms = si
            .proximity_map_with_geos(big.clone())
            .unwrap()
            .collect::<Vec<_>>();
        actual_geoms.sort();
        assert_eq!(actual_geoms, expected_geoms);
    }

    #[cfg(feature = "parallel")]
    pub fn test_par_prox_map<Small, Big, E1, E2>(
        config: Config,
        small: Small,
        big: Big,
        expected: &Vec<ProxMapRow>,
    ) where
        Small: TryInto<Par<SplitGeoSeq>, Error = E1> + Clone,
        Big: TryInto<Par<SplitGeoSeq>, Error = E2> + Clone,
        E1: std::any::Any + std::fmt::Debug,
        E2: std::any::Any + std::fmt::Debug,
    {
        let small_geoms = par_sgs_try_into(small.clone())
            .expect("small conversion")
            .to_vec();
        let big_geoms = par_sgs_try_into(big.clone())
            .expect("big conversion")
            .to_vec();
        let expected_geoms: Vec<_> = expected
            .iter()
            .map(|pmr| ProxMapGeoRow {
                big_index: pmr.big_index,
                small_index: pmr.small_index,
                distance: pmr.distance,
                big: big_geoms[pmr.big_index].clone(),
                small: small_geoms[pmr.small_index].clone(),
            })
            .collect();
        let _expected_geoms2 = expected_geoms.clone();

        let si = config
            .clone()
            .parallel(small.clone())
            .expect("construction succeeded");
        let mut actual = si.proximity_map(big.clone()).unwrap().collect::<Vec<_>>();
        actual.sort();
        assert_eq!(actual, *expected);

        let mut actual_geoms = si
            .proximity_map_with_geos(big.clone())
            .unwrap()
            .collect::<Vec<_>>();
        actual_geoms.sort();
        assert_eq!(actual_geoms, expected_geoms);
    }

    pub fn test_spatial_join<Small, Big, E1, E2>(
        config: Config,
        small: Small,
        big: Big,
        interaction: Interaction,
        expected: &Vec<SJoinRow>,
    ) where
        Small: TryInto<SplitGeoSeq, Error = E1> + Clone,
        Big: TryInto<SplitGeoSeq, Error = E2> + Clone,
        E1: std::any::Any + std::fmt::Debug,
        E2: std::any::Any + std::fmt::Debug,
    {
        let small_geoms = sgs_try_into(small.clone())
            .expect("small conversion")
            .to_vec();
        let big_geoms = sgs_try_into(big.clone()).expect("big conversion").to_vec();
        let expected_geoms: Vec<_> = expected
            .iter()
            .map(|sjr| SJoinGeoRow {
                big_index: sjr.big_index,
                small_index: sjr.small_index,
                big: big_geoms[sjr.big_index].clone(),
                small: small_geoms[sjr.small_index].clone(),
            })
            .collect();
        let _expected_geoms2 = expected_geoms.clone();

        let si = config
            .clone()
            .serial(small.clone())
            .expect("construction succeeded");
        let mut actual = si
            .spatial_join(big.clone(), interaction)
            .unwrap()
            .collect::<Vec<_>>();
        actual.sort();
        assert_eq!(actual, *expected);

        let mut actual_geoms = si
            .spatial_join_with_geos(big.clone(), interaction)
            .unwrap()
            .collect::<Vec<_>>();
        actual_geoms.sort();
        assert_eq!(actual_geoms, expected_geoms);
    }

    #[cfg(feature = "parallel")]
    pub fn test_par_spatial_join<Small, Big, E1, E2>(
        config: Config,
        small: Small,
        big: Big,
        interaction: Interaction,
        expected: &Vec<SJoinRow>,
    ) where
        Small: TryInto<Par<SplitGeoSeq>, Error = E1> + Clone,
        Big: TryInto<Par<SplitGeoSeq>, Error = E2> + Clone,
        E1: std::any::Any + std::fmt::Debug,
        E2: std::any::Any + std::fmt::Debug,
    {
        let small_geoms = par_sgs_try_into(small.clone())
            .expect("small conversion")
            .to_vec();
        let big_geoms = par_sgs_try_into(big.clone())
            .expect("big conversion")
            .to_vec();
        let expected_geoms: Vec<_> = expected
            .iter()
            .map(|sjr| SJoinGeoRow {
                big_index: sjr.big_index,
                small_index: sjr.small_index,
                big: big_geoms[sjr.big_index].clone(),
                small: small_geoms[sjr.small_index].clone(),
            })
            .collect();
        let _expected_geoms2 = expected_geoms.clone();

        let si = config
            .clone()
            .parallel(small.clone())
            .expect("construction succeeded");
        let mut actual = si
            .spatial_join(big.clone(), interaction)
            .unwrap()
            .collect::<Vec<_>>();
        actual.sort();
        assert_eq!(actual, *expected);

        let mut actual_geoms = si
            .spatial_join_with_geos(big.clone(), interaction)
            .unwrap()
            .collect::<Vec<_>>();
        actual_geoms.sort();
        assert_eq!(actual_geoms, expected_geoms);
    }

    #[test]
    fn simple_index_self() {
        let config = Config::new().max_distance(4.);
        let small = vec![Point::new(1., 1.)];
        let big = vec![Point::new(1., 1.)];
        let expected = vec![ProxMapRow {
            big_index: 0,
            small_index: 0,
            distance: 0.,
        }];
        test_prox_map(config, small.clone(), big.clone(), &expected);
        #[cfg(feature = "parallel")]
        test_par_prox_map(config, small, big, &expected);
    }

    #[test]
    fn self_spatial_join_pair() {
        let config = Config::new();
        let pts = vec![
            geo::Geometry::Point(Point::new(1., 1.)),
            geo::Geometry::Point(Point::new(22., 22.)),
        ];
        let expected = vec![
            SJoinRow {
                big_index: 0,
                small_index: 0,
            },
            SJoinRow {
                big_index: 1,
                small_index: 1,
            },
        ];
        test_spatial_join(config, &pts, &pts, Interaction::Intersects, &expected);
        #[cfg(feature = "parallel")]
        test_par_spatial_join(config, &pts, &pts, Interaction::Intersects, &expected);
    }

    #[test]
    fn simple_index_some_other() {
        let config = Config::new().max_distance(4.);
        let small = vec![Point::new(1., 1.)];
        let big = vec![Point::new(2., 1.)];
        let expected = vec![ProxMapRow {
            big_index: 0,
            small_index: 0,
            distance: 1.0,
        }];
        test_prox_map(config, small.clone(), big.clone(), &expected);
        #[cfg(feature = "parallel")]
        test_par_prox_map(config, small, big, &expected);
    }

    #[test]
    fn simple_index_none() {
        let config = Config::new().max_distance(0.5);
        let small = vec![Point::new(1., 1.)];
        let big = vec![Point::new(2., 1.)];
        let expected = vec![];
        test_prox_map(config, small.clone(), big.clone(), &expected);
        #[cfg(feature = "parallel")]
        test_par_prox_map(config, small, big, &expected);
    }
    // for all pairs of types, verift that prox map finds and doesn't find depending on max_distance
}
