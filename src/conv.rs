use std::convert::TryFrom;

use geo::{Geometry, Line, LineString, Point, Polygon, Rect, Triangle};
#[cfg(feature = "parallel")]
use rayon::prelude::*;

use super::validation::IsSafe;
use crate::{structs::Par, Error, Indexes, SplitGeo, SplitGeoIndexes, SplitGeoSeq};

impl TryFrom<&[Geometry<f64>]> for SplitGeoSeq {
    type Error = Error;

    fn try_from(seq: &[Geometry<f64>]) -> Result<Self, Self::Error> {
        let mut result = SplitGeoSeq::default();
        for (i, geo) in seq.iter().enumerate() {
            match geo {
                Geometry::Line(ln) => {
                    ln.is_safe(i)?;
                    result.geos.lines.push(*ln);
                    result.indexes.lines.push(i);
                }
                Geometry::Point(pt) => {
                    pt.is_safe(i)?;
                    result.geos.points.push(*pt);
                    result.indexes.points.push(i)
                }
                Geometry::Polygon(poly) => {
                    poly.is_safe(i)?;
                    result.geos.polys.push(poly.clone());
                    result.indexes.polys.push(i)
                }
                Geometry::LineString(ls) => {
                    ls.is_safe(i)?;
                    result.geos.line_strings.push(ls.clone());
                    result.indexes.line_strings.push(i)
                }
                Geometry::Rect(r) => {
                    r.is_safe(i)?;
                    result.geos.rects.push(*r);
                    result.indexes.rects.push(i)
                }
                Geometry::Triangle(tri) => {
                    tri.is_safe(i)?;
                    result.geos.tris.push(*tri);
                    result.indexes.tris.push(i)
                }

                _ => unimplemented!("ugh"),
            }
        }
        Ok(result)
    }
    // FIXME: add an optimization that looks for cases where all but
    // one variants are empty and makes them implicit.
}

#[cfg(feature = "parallel")]
impl TryFrom<SplitGeoSeq> for Par<SplitGeoSeq> {
    type Error = Error;
    fn try_from(sgs: SplitGeoSeq) -> Result<Self, Self::Error> {
        Ok(Par(sgs))
    }
}

#[cfg(feature = "parallel")]
impl TryFrom<&[Geometry<f64>]> for Par<SplitGeoSeq> {
    type Error = Error;

    fn try_from(seq: &[Geometry<f64>]) -> Result<Self, Self::Error> {
        let step = (seq.len() / num_cpus::get()).max(1);
        let end = seq.len();
        (0..seq.len())
            .into_par_iter()
            .step_by(step)
            .map(|start| std::ops::Range {
                start,
                end: (start + step).min(end),
            })
            .map(|range| {
                let offset = range.start;
                SplitGeoSeq::try_from(&seq[range]).map(|sgs| {
                    sgs.indexes.points.add_offset(offset);
                    sgs.indexes.lines.add_offset(offset);
                    sgs.indexes.polys.add_offset(offset);
                    sgs.indexes.line_strings.add_offset(offset);
                    sgs.indexes.rects.add_offset(offset);
                    sgs.indexes.tris.add_offset(offset);

                    sgs
                })
            })
            .try_reduce(SplitGeoSeq::default, |a, b| Ok(SplitGeoSeq::merge(a, b)))
            .map(Par)
    }
}

// FIXME: make versions of these two that consume the arg (i.e., Vec instead of &Vec)
impl TryFrom<&Vec<Geometry<f64>>> for SplitGeoSeq {
    type Error = Error;

    fn try_from(seq: &Vec<Geometry<f64>>) -> Result<Self, Self::Error> {
        SplitGeoSeq::try_from(&seq[..])
    }
}

impl TryFrom<Vec<Geometry<f64>>> for SplitGeoSeq {
    type Error = Error;

    fn try_from(seq: Vec<Geometry<f64>>) -> Result<Self, Self::Error> {
        SplitGeoSeq::try_from(&seq)
    }
}

impl TryFrom<&geo::GeometryCollection<f64>> for SplitGeoSeq {
    type Error = Error;

    fn try_from(seq: &geo::GeometryCollection<f64>) -> Result<Self, Self::Error> {
        SplitGeoSeq::try_from(&seq.0[..])
    }
}

#[cfg(feature = "parallel")]
impl TryFrom<&Vec<Geometry<f64>>> for Par<SplitGeoSeq> {
    type Error = Error;

    fn try_from(seq: &Vec<Geometry<f64>>) -> Result<Self, Self::Error> {
        Par::<SplitGeoSeq>::try_from(&seq[..])
    }
}

#[cfg(feature = "parallel")]
impl TryFrom<Vec<Geometry<f64>>> for Par<SplitGeoSeq> {
    type Error = Error;

    fn try_from(seq: Vec<Geometry<f64>>) -> Result<Self, Self::Error> {
        Par::<SplitGeoSeq>::try_from(&seq)
    }
}

#[cfg(feature = "parallel")]
impl TryFrom<&geo::GeometryCollection<f64>> for Par<SplitGeoSeq> {
    type Error = Error;

    fn try_from(seq: &geo::GeometryCollection<f64>) -> Result<Self, Self::Error> {
        Par::<SplitGeoSeq>::try_from(&seq.0[..])
    }
}

macro_rules! static_cond {
    (true, $consequent:expr, $alternative:expr) => {
        $consequent
    };
    (false, $consequent:expr, $alternative:expr) => {
        $alternative
    };
}

// make conversions from &Vec, Vec, and slice
macro_rules! from_impls {
    ($ItemType:ty, $Var:ident, $IsCopyable:ident) => {
        impl TryFrom<&[$ItemType]> for SplitGeoSeq {
            type Error = Error;

            fn try_from(seq: &[$ItemType]) -> Result<Self, Self::Error> {
                seq.iter()
                    .enumerate()
                    .try_for_each(|(i, x)| (*x).is_safe(i))
                    .map(|_| SplitGeoSeq {
                        geos: SplitGeo {
                            $Var: static_cond!(
                                $IsCopyable,
                                seq.to_vec(),
                                seq.iter().map(|poly| poly.clone()).collect()
                            ),
                            ..Default::default()
                        },
                        indexes: SplitGeoIndexes {
                            $Var: Indexes::Range(0..seq.len()),
                            ..Default::default()
                        },
                    })
            }
        }

        #[cfg(feature = "parallel")]
        impl TryFrom<&[$ItemType]> for Par<SplitGeoSeq> {
            type Error = Error;

            fn try_from(seq: &[$ItemType]) -> Result<Self, Self::Error> {
                seq.par_iter()
                    .enumerate()
                    .try_for_each(|(i, x)| (*x).is_safe(i))
                    .map(|_| {
                        Par(SplitGeoSeq {
                            geos: SplitGeo {
                                $Var: static_cond!(
                                    $IsCopyable,
                                    seq.to_vec(),
                                    seq.iter().map(|poly| poly.clone()).collect()
                                ),
                                ..Default::default()
                            },
                            indexes: SplitGeoIndexes {
                                $Var: Indexes::Range(0..seq.len()),
                                ..Default::default()
                            },
                        })
                    })
            }
        }

        impl TryFrom<Vec<$ItemType>> for SplitGeoSeq {
            type Error = Error;

            fn try_from(seq: Vec<$ItemType>) -> Result<Self, Self::Error> {
                seq.iter()
                    .enumerate()
                    .try_for_each(|(i, x)| (*x).is_safe(i))
                    .map(|_| SplitGeoSeq {
                        indexes: SplitGeoIndexes {
                            $Var: Indexes::Range(0..seq.len()),
                            ..Default::default()
                        },
                        geos: SplitGeo {
                            $Var: seq,
                            ..Default::default()
                        },
                    })
            }
        }

        #[cfg(feature = "parallel")]
        impl TryFrom<Vec<$ItemType>> for Par<SplitGeoSeq> {
            type Error = Error;

            fn try_from(seq: Vec<$ItemType>) -> Result<Self, Self::Error> {
                seq.par_iter()
                    .enumerate()
                    .try_for_each(|(i, x)| (*x).is_safe(i))
                    .map(|_| {
                        Par(SplitGeoSeq {
                            indexes: SplitGeoIndexes {
                                $Var: Indexes::Range(0..seq.len()),
                                ..Default::default()
                            },
                            geos: SplitGeo {
                                $Var: seq,
                                ..Default::default()
                            },
                        })
                    })
            }
        }

        impl TryFrom<&Vec<$ItemType>> for SplitGeoSeq {
            type Error = Error;

            fn try_from(seq: &Vec<$ItemType>) -> Result<Self, Self::Error> {
                SplitGeoSeq::try_from(&seq[..])
            }
        }

        #[cfg(feature = "parallel")]
        impl TryFrom<&Vec<$ItemType>> for Par<SplitGeoSeq> {
            type Error = Error;

            fn try_from(seq: &Vec<$ItemType>) -> Result<Self, Self::Error> {
                Par::<SplitGeoSeq>::try_from(&seq[..])
            }
        }
    };
}

from_impls!(Point<f64>, points, true);
from_impls!(Line<f64>, lines, true);
from_impls!(Polygon<f64>, polys, false);
from_impls!(LineString<f64>, line_strings, false);
from_impls!(Rect<f64>, rects, true);
from_impls!(Triangle<f64>, tris, true);

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn from_geos() {
        let pt: Point<f64> = (1.1, 2.2).into();
        let ln1: Line<f64> = Line::new((0., 0.), (1., 1.));
        let ln2: Line<f64> = Line::new((2., 2.), (1., 1.));
        let poly = Polygon::new(
            geo::LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
            vec![],
        );
        let expected = Ok(SplitGeoSeq {
            geos: SplitGeo {
                points: vec![pt],
                lines: vec![ln1, ln2],
                polys: vec![poly.clone()],
                line_strings: vec![],
                rects: vec![],
                tris: vec![],
            },
            indexes: SplitGeoIndexes {
                points: Indexes::Range(0..1),
                lines: Indexes::Explicit(vec![1, 3]),
                polys: Indexes::Explicit(vec![2]),
                line_strings: Indexes::default(),
                rects: Indexes::default(),
                tris: Indexes::default(),
            },
        });
        let geos = vec![
            Geometry::Point(pt),
            Geometry::Line(ln1),
            Geometry::Polygon(poly.clone()),
            Geometry::Line(ln2),
        ];
        let slice_result = SplitGeoSeq::try_from(&geos[..]);
        let vec_result = SplitGeoSeq::try_from(&geos);
        let geo_collection_result = SplitGeoSeq::try_from(&geo::GeometryCollection(geos));
        assert_eq!(expected, slice_result);
        assert_eq!(expected, vec_result);
        assert_eq!(expected, geo_collection_result);
    }

    #[test]
    fn from_points() {
        let pt: Point<f64> = (1.1, 2.2).into();
        let pts = vec![pt, pt, pt];
        let expected = Ok(SplitGeoSeq {
            geos: SplitGeo {
                points: pts.clone(),
                ..Default::default()
            },
            indexes: SplitGeoIndexes {
                points: Indexes::Range(0..3),
                ..Default::default()
            },
        });
        let slice_result = SplitGeoSeq::try_from(&pts[..]);
        let vec_result = SplitGeoSeq::try_from(&pts);
        let vec_into_result = SplitGeoSeq::try_from(pts);
        assert_eq!(expected, slice_result);
        assert_eq!(expected, vec_result);
        assert_eq!(expected, vec_into_result);
    }
}
