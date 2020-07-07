[![spatial-join on Crates.io](https://meritbadge.herokuapp.com/spatial-join)](https://crates.io/crates/spatial-join)
[![Docs.rs](https://docs.rs/spatial-join/badge.svg)](https://docs.rs/spatial-join)

`spatial-join` provides tools to perform streaming geospatial-joins on geographic data.

## Documentation

Check out [docs at docs.rs](https://docs.rs/spatial-join)


## Spatial Joins

Given two sequences of geospatial shapes, `small` and `big`, a
spatial-join indicates which elements of `small` and `big`
intersect. You could compute this yourself using a nested loop,
but like any good spatial-join package, this one uses
[R-trees](https://en.wikipedia.org/wiki/R-tree) to dramatically
reduce the search space.

We're not limited to intersections only! We can also find pairs
where elements of `small` contain elements of `big` or are within
elements of `big` by passing different values of
[Interaction](https://docs.rs/spatial-join/latest/spatial_join/enum.Interaction.html).

## Proximity Maps

While spatial join is a well known term, proximity map is
not. Given two sequences of shapes `small` and `big`, it just
finds all pairs of items whose distance is less than some
threshold. You set that threshold using the
[`max_distance`](https://docs.rs/spatial-join/latest/spatial_join/struct.Config.html#method.max_distance) method
on the [`Config`](https://docs.rs/spatial-join/latest/spatial_join/struct.Config.html) struct.

## Inputs

Inputs are sequences of shapes, and shapes must be one of the
following elements from the
[`geo`](https://docs.rs/geo/latest/geo/) crate:
* [points](https://docs.rs/geo/latest/geo/struct.Point.html),
* [lines](https://docs.rs/geo/latest/geo/struct.Line.html),
* [line strings](https://docs.rs/geo/latest/geo/struct.LineString.html),
* [polygons](https://docs.rs/geo/latest/geo/struct.Polygon.html),
* [rectangles](https://docs.rs/geo/latest/geo/struct.Rect.html),
* [triangles](https://docs.rs/geo/latest/geo/struct.Triangle.html), or
* the [Geometry](https://docs.rs/geo/latest/geo/enum.Geometry.html) enum

`MultiPoint`, `MultiLineString`, and `MultiPolygon` are *not* supported.

While the [geo] crate makes these types generic over the
coordinate type, `spatial-join` only supports [geo] types
parametrized with [std::f64] coordinate types (i.e.,
`Polygon<f64>`).

So what kind of sequences can you use?
* slices: `&[T]`,
* vectors: `Vec<T>` or `&Vec<T>`, or
* [`&geo::GeometryCollection`](https://docs.rs/geo/latest/geo/struct.GeometryCollection.html)

In addition:
* all coordinate values must be finite
* `LineStrings` must have at least two points
* `Polygon` exteriors must have at least three points

Input that doesn't meet these conditions will return an [error](https://docs.rs/spatial-join/latest/spatial_join/enum.Error.html).

## Outputs

[`SpatialIndex::spatial_join`](https://docs.rs/spatial-join/latest/spatial_join/struct.SpatialIndex.html#method.spatial_join) returns `Result<impl
Iterator<Item=SJoinRow>, Error>` where
[`SJoinRow`](https://docs.rs/spatial-join/latest/spatial_join/struct.SJoinRow.html) gives you indexes into
`small` and `big` to find the corresponding geometries.

Alternatively, you can use [`SpatialIndex::spatial_join_with_geos`](https://docs.rs/spatial-join/latest/spatial_join/struct.SpatialIndex.html#method.spatial_join_with_geos)
which returns `Result<impl Iterator<Item=SJoinGeoRow>, Error>`.
[`SJoinGeoRow`](https://docs.rs/spatial-join/latest/spatial_join/struct.SJoinGeoRow.html) differs from
[`SJoinRow`](https://docs.rs/spatial-join/latest/spatial_join/struct.SJoinRow.html) only in the addition of `big`
and `small`
[`Geometry`](https://docs.rs/geo/latest/geo/enum.Geometry.html)
fields so you can work directly with the source geometries without
having to keep the original sequences around. This convenience
comes at the cost of cloning the source geometries which can be
expensive for geometries that use heap storage like `LineString`
and `Polygon`.

In a similar manner, [`SpatialIndex::proximity_map`](https://docs.rs/spatial-join/latest/spatial_join/struct.SpatialIndex.html#method.proximity_map) and
[`SpatialIndex::proximity_map_with_geos`](https://docs.rs/spatial-join/latest/spatial_join/struct.SpatialIndex.html#method.proximity_map) offer
[`ProxMapRow`](https://docs.rs/spatial-join/latest/spatial_join/struct.ProxMapRow.html) and
[`ProxMapGeoRow`](https://docs.rs/spatial-join/latest/spatial_join/struct.ProxMapGeoRow.html) iterators in their
return types. These differ from their `SJoin` counterparts only in
the addition of a `distance` field.

## Examples

Here's the simplest thing: let's verify that a point intersects itself.
```
use spatial_join::*;
use geo::{Geometry, Point};
// Create a new spatial index loaded with just one point
let idx = Config::new()
    // Ask for a serial index that will process data on only one core
    .serial(vec![Geometry::Point(Point::new(1.1, 2.2))])
    .unwrap(); // Creating an index can fail!
let results: Vec<_> = idx
    .spatial_join(
        vec![Geometry::Point(Point::new(1.1, 2.2))],
        Interaction::Intersects,
    )
    .unwrap() // spatial_join can fail, but we'll assume it won't here
    .collect(); // we actually get an iterator, but let's collect it into a Vector.
assert_eq!(
    results,
    vec![SJoinRow {
        big_index: 0,
        small_index: 0
    }]
);
```

For a slightly more complicated, we'll take a box and a smaller
box and verify that the big box contains the smaller box, and
we'll do it all in parallel.
```
use spatial_join::*;
use geo::{Coordinate, Geometry, Point, Rect};
   use rayon::prelude::*;

let idx = Config::new()
    .parallel(vec![Geometry::Rect(Rect::new(
        Coordinate { x: -1., y: -1. },
        Coordinate { x: 1., y: 1. },
    ))])
    .unwrap();
let results: Vec<_> = idx
    .spatial_join(
        vec![Geometry::Rect(Rect::new(
            Coordinate { x: -0.5, y: -0.5 },
            Coordinate { x: 0.5, y: 0.5 },
    ))],
        Interaction::Contains,
    )
    .unwrap()
    .collect();
assert_eq!(
    results,
    vec![SJoinRow {
        big_index: 0,
        small_index: 0
    }]
);
```

## Crate Features

- `parallel`
  - Enabled by default.
  - This adds a dependency on
    [`rayon`](https://crates.io/crates/rayon) and provides a
    [`parallel`](https://docs.rs/spatial-join/latest/spatial_join/struct.Config.html#method.parallel) method that
    returns a [`ParSpatialIndex`](https://docs.rs/spatial-join/latest/spatial_join/struct.ParSpatialIndex.html)
    just like the [`SpatialIndex`](https://docs.rs/spatial-join/latest/spatial_join/struct.SpatialIndex.html)
    that [`serial`](https://docs.rs/spatial-join/latest/spatial_join/struct.Config.html#method.serial) returns
    except that all the methods return `Result<impl
    ParallelIterator>` instead of `Result<impl Iterator>`.

## Geographic

Right now, this entire crate assumes that you're dealing with
euclidean geometry on a two-dimensional plane. But that's unusual:
typically you've got geographic coordinates (longitude and
latitude measured in decimal degrees). To use the tools in this
package correctly, you should really reproject your geometries
into an appropriate euclidean coordinate system. That might be
require you to do a lot of extra work if the extent of your
geometry sets exceeds what any reasonable projection can handle.

Alternatively, you can just pretend that geodetic coordinates are
euclidean. For spatial-joins that will mostly work if all of your
geometries steer well-clear of the anti-meridian (longitude=±180
degrees) and the polar regions as well.

For proximity maps, you'll need to pick an appropriate
`max_distance` value measured in decimal degrees which will be
used for both longitude and latitude offsets
simulataneously. That's challenging because while one degree of
latitude is always the same (about 110 km), one degree of
longitude changes from about 110 km at the equator to 0 km at the
poles. If your geometry sets have a narrow extant and are near the
equator, you might be able to find a `max_distance` value that
works, but that's pretty unlikely.

## Performance

* You'll notice that our API specifies geometry sequences in terms
  of `small` and `big`. In order to construct a spatial index
  object, we have to build a series of R-trees, one per geometry
  type, using bulk loading. This process is expensive
  (`O(n*log(n))`) so you'll probably get better overall performance
  if you index the smaller sequence.
* Because the spatial-join and proximity-map operations are
  implemented as iterators, you can process very large data-sets
  with low memory usage. But you do need to keep both the `small`
  and `large` geometry sequence in memory, in addition to rtrees
  for the `small` sequence. Note that in some cases, specifically
  whenever we're processing a heap-bound element of the `large`
  sequence (i.e., Polygons or LineStrings), we will buffer all
  matching result records for each such `large` geometry.
* If you use a non-zero `max_distance` value, then any
  spatial-join operations will be somewhat slower since
  `max_distance` effectively buffers `small` geometries in the
  r-trees. You'll still get the correct answer, but it might take
  longer. The larger the `max_distance` value, the longer it will
  take.


## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
