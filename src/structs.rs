use std::convert::TryInto;

use geo::{Geometry, Line, LineString, Point, Polygon, Rect, Triangle};
use thiserror::Error;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum Error {
    #[error("Inifinite or NaN coordinate value in geo at index {0:?}: {1:?}")]
    BadCoordinateValue(usize, Geometry<f64>),

    #[error("max_distance must be finite and greater than or equal to zero: {0:?}")]
    BadMaxDistance(f64),

    #[error("LineString at index {0:?} must have at least two points")]
    LineStringTooSmall(usize),

    #[error("Polygon at index {0:?} must have an exterior with at least three points")]
    PolygonExteriorTooSmall(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interaction {
    Intersects,
    Within,
    Contains,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Config {
    pub max_distance: f64,
}

impl Config {
    pub fn new() -> Config {
        Default::default()
    }

    #[allow(clippy::needless_update)]
    pub fn max_distance(self, value: f64) -> Config {
        Config {
            max_distance: value,
            ..self
        }
    }

    pub fn validate(&self) -> Option<Error> {
        if !(self.max_distance.is_finite() && self.max_distance >= 0.) {
            return Some(Error::BadMaxDistance(self.max_distance));
        }

        None
    }

    pub fn serial<T, U>(self, small: T) -> Result<super::SpatialIndex, Error>
    where
        T: TryInto<SplitGeoSeq, Error = U>,
        U: std::any::Any,
    {
        if let Some(error) = self.validate() {
            return Err(error);
        }
        super::SpatialIndex::new(small, self)
    }

    #[cfg(feature = "parallel")]
    pub fn parallel<T, U>(self, small: T) -> Result<super::ParSpatialIndex, Error>
    where
        T: TryInto<Par<SplitGeoSeq>, Error = U>,
        U: std::any::Any,
    {
        if let Some(error) = self.validate() {
            return Err(error);
        }
        super::ParSpatialIndex::new(small, self)
    }
}

pub struct Par<T>(pub T);

#[derive(Default, PartialEq, Debug, Clone)]
pub(crate) struct SplitGeo {
    pub points: Vec<Point<f64>>,
    pub lines: Vec<Line<f64>>,
    pub polys: Vec<Polygon<f64>>,
    pub line_strings: Vec<LineString<f64>>,
    pub rects: Vec<Rect<f64>>,
    pub tris: Vec<Triangle<f64>>,
}

impl SplitGeoSeq {
    pub fn merge(mut a: SplitGeoSeq, mut b: SplitGeoSeq) -> SplitGeoSeq {
        a.geos.points.append(&mut b.geos.points);
        a.geos.lines.append(&mut b.geos.lines);
        a.geos.polys.append(&mut b.geos.polys);
        a.geos.line_strings.append(&mut b.geos.line_strings);
        a.geos.rects.append(&mut b.geos.rects);
        a.geos.tris.append(&mut b.geos.tris);

        a.indexes.points = a.indexes.points.merge(b.indexes.points);
        a.indexes.lines = a.indexes.lines.merge(b.indexes.lines);
        a.indexes.line_strings = a.indexes.line_strings.merge(b.indexes.line_strings);
        a.indexes.polys = a.indexes.polys.merge(b.indexes.polys);
        a.indexes.rects = a.indexes.rects.merge(b.indexes.rects);
        a.indexes.tris = a.indexes.tris.merge(b.indexes.tris);

        a
    }
}

lazy_static::lazy_static! {
    static ref EMPTY_VEC: Vec<usize> = vec![];
}

// Sigh...maybe we should just replace this with IDLBitRange
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Indexes {
    Explicit(Vec<usize>),
    Range(std::ops::Range<usize>),
}

impl Default for Indexes {
    fn default() -> Self {
        Indexes::Range(0..0)
    }
}
impl Indexes {
    pub fn push(&mut self, index: usize) {
        match self {
            Indexes::Explicit(v) => {
                if let Some(last) = v.last() {
                    assert!(*last <= index);
                }
                v.push(index);
            }
            Indexes::Range(r) => {
                if r.end == index {
                    r.end = index + 1;
                } else {
                    let mut v: Vec<usize> = r.collect();
                    v.push(index);
                    *self = Indexes::Explicit(v);
                }
            }
        }
    }

    fn range(&self) -> std::ops::Range<usize> {
        match self {
            Indexes::Range(r) => r.clone(),
            _ => std::ops::Range {
                start: 0 as usize,
                end: 0 as usize,
            },
        }
    }

    // The one user for this method is test code but it would be a
    // pain to extract it to live with the rest of the test code.
    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.range().chain(match self {
            Indexes::Range(_) => EMPTY_VEC.iter().copied(),
            Indexes::Explicit(v) => v.iter().copied(),
        })
    }

    pub fn into_iter(self) -> impl Iterator<Item = usize> {
        self.range().chain(match self {
            Indexes::Range(_) => vec![].into_iter(),
            Indexes::Explicit(v) => v.into_iter(),
        })
    }

    #[cfg(feature = "parallel")]
    pub fn into_par_iter(self) -> impl rayon::iter::IndexedParallelIterator<Item = usize> {
        self.range().into_par_iter().chain(match self {
            Indexes::Range(_) => vec![].into_par_iter(),
            Indexes::Explicit(v) => v.into_par_iter(),
        })
    }

    pub fn get(&self, index: usize) -> usize {
        match self {
            Indexes::Range(r) => index + r.start,
            Indexes::Explicit(v) => v[index],
        }
    }

    #[cfg(feature = "parallel")]
    pub fn add_offset(&mut self, offset: usize) {
        let new = match self {
            Indexes::Range(r) => Indexes::Range(if r.start == r.end {
                r.clone()
            } else {
                std::ops::Range {
                    start: r.start + offset,
                    end: r.end + offset,
                }
            }),
            Indexes::Explicit(v) => {
                Indexes::Explicit(v.iter().map(|value| value + offset).collect())
            }
        };
        *self = new;
    }

    pub fn canonicalize(&mut self) {
        if let Indexes::Explicit(v) = self {
            if v.is_empty() {
                *self = Indexes::Range(0..0);
            } else {
                let is_contiguous = v.windows(2).all(|slice| (slice[1] - slice[0]) == 1);
                if is_contiguous {
                    *self = Indexes::Range(v[0]..(v[v.len() - 1] + 1));
                }
            }
        }
    }

    pub fn merge(mut self, mut other: Indexes) -> Indexes {
        // This is more complicated than it should be because rayon's
        // reduce makes no guarantees about order.

        fn is_empty(r: &std::ops::Range<usize>) -> bool {
            // not in stable yet, sigh
            r.start == r.end
        }

        fn join_ranges(a: std::ops::Range<usize>, b: std::ops::Range<usize>) -> Indexes {
            if is_empty(&a) {
                return Indexes::Range(b);
            };
            if is_empty(&b) {
                return Indexes::Range(a);
            };

            if a.end == b.start {
                Indexes::Range(a.start..b.end)
            } else if b.end == a.start {
                Indexes::Range(b.start..a.end)
            } else {
                let (a, b) = if a.end < b.start { (a, b) } else { (b, a) };
                Indexes::Explicit(a.chain(b).collect())
            }
        }

        fn join_vec(mut a: Vec<usize>, mut b: Vec<usize>) -> Indexes {
            if a.is_empty() {
                return Indexes::Explicit(b);
            }
            if b.is_empty() {
                return Indexes::Explicit(a);
            }

            Indexes::Explicit(if a[0] <= b[0] {
                a.append(&mut b);
                a
            } else {
                b.append(&mut a);
                b
            })
        }

        fn join_range_vec(a: std::ops::Range<usize>, b: Vec<usize>) -> Indexes {
            if b.is_empty() {
                // Do b first because if they're both empty, we prefer
                // a Range representation.
                return Indexes::Range(a);
            }
            if is_empty(&a) {
                return Indexes::Explicit(b);
            }

            Indexes::Explicit(if a.end <= b[0] {
                a.chain(b.into_iter()).collect()
            } else {
                b.into_iter().chain(a).collect()
            })
        }

        self.canonicalize();
        other.canonicalize();
        let mut res = match (self, other) {
            (Indexes::Range(a), Indexes::Range(b)) => join_ranges(a, b),
            (Indexes::Range(a), Indexes::Explicit(b)) => join_range_vec(a, b),
            (Indexes::Explicit(a), Indexes::Range(b)) => join_range_vec(b, a),
            (Indexes::Explicit(a), Indexes::Explicit(b)) => join_vec(a, b),
        };
        res.canonicalize();
        res
    }
}

#[derive(Default, PartialEq, Debug, Clone)]
pub(crate) struct SplitGeoIndexes {
    pub points: Indexes,
    pub lines: Indexes,
    pub polys: Indexes,
    pub line_strings: Indexes,
    pub rects: Indexes,
    pub tris: Indexes,
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct SplitGeoSeq {
    pub(crate) geos: SplitGeo,
    pub(crate) indexes: SplitGeoIndexes,
}

#[derive(Clone, Copy, Debug)]
pub struct ProxMapRow {
    pub big_index: usize,
    pub small_index: usize,
    pub distance: f64,
}

impl Eq for ProxMapRow {}

impl PartialEq for ProxMapRow {
    fn eq(&self, other: &Self) -> bool {
        (self.big_index, self.small_index) == (other.big_index, other.small_index)
    }
}

impl PartialOrd for ProxMapRow {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProxMapRow {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.big_index, self.small_index).cmp(&(other.big_index, other.small_index))
    }
}

#[derive(Clone, Debug)]
pub struct ProxMapGeoRow {
    pub big_index: usize,
    pub small_index: usize,
    pub big: Geometry<f64>,
    pub small: Geometry<f64>,
    pub distance: f64,
}

impl Eq for ProxMapGeoRow {}

impl PartialEq for ProxMapGeoRow {
    fn eq(&self, other: &Self) -> bool {
        (self.big_index, self.small_index) == (other.big_index, other.small_index)
    }
}

impl PartialOrd for ProxMapGeoRow {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProxMapGeoRow {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.big_index, self.small_index).cmp(&(other.big_index, other.small_index))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct SJoinRow {
    pub big_index: usize,
    pub small_index: usize,
}

#[derive(Clone, Debug)]
pub struct SJoinGeoRow {
    pub big_index: usize,
    pub small_index: usize,
    pub big: Geometry<f64>,
    pub small: Geometry<f64>,
}

impl Eq for SJoinGeoRow {}

impl PartialEq for SJoinGeoRow {
    fn eq(&self, other: &Self) -> bool {
        (self.big_index, self.small_index) == (other.big_index, other.small_index)
    }
}

impl PartialOrd for SJoinGeoRow {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SJoinGeoRow {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.big_index, self.small_index).cmp(&(other.big_index, other.small_index))
    }
}
